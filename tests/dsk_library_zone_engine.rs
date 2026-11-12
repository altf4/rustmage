//! Committed library-to-graveyard batches and optional grouped shuffles.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const LAND_RECOVERY: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LandCardsEnterYourGraveyardFromLibrary,
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnGraveyardArrivalBatch { tapped: true }],
    }],
    ..CardRules::EMPTY
};
const MILL_BOTH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::Mill(PlayerSet::Each, MillAmount::Fixed(3))],
    ),
    ..CardRules::EMPTY
};
const fn graveyard_card(relation: i8) -> TargetRule {
    TargetRule::Object {
        zone: TargetZone::Graveyard,
        kinds: 0,
        relation,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }
}
const EXILE_GRAVEYARD_CARD: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(graveyard_card(-1))),
        &[Instruction::Exile],
    ),
    ..CardRules::EMPTY
};

const SHUFFLE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 16 | 32,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            TargetSpec {
                rule: graveyard_card(0),
                min: 0,
                max: 4,
            },
        ]),
        &[
            Instruction::ShuffleTargetIntoLibrary,
            Instruction::MayPlayer {
                player: PlayerSet::Controller,
                skip: 1,
            },
            Instruction::ShuffleGraveyardTargetsIntoControllerLibrary { cards_group: 1 },
        ],
    ),
    ..CardRules::EMPTY
};

const GRAVEYARD_DEPARTURE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CardsLeaveGraveyard,
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

fn pass_twice(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn the_committed_mill_batch_returns_only_its_exact_surviving_lands_tapped() {
    let opt = Card::from_name("Opt").unwrap();
    let growth = Card::from_name("Giant Growth").unwrap();
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &LAND_RECOVERY),
        (opt, &MILL_BOTH),
        (growth, &EXILE_GRAVEYARD_CARD),
    ]);
    let player = Player::P0;
    let opponent = player.other();
    let mut parent = scenario(player);
    let observer = put(&mut parent, player, "Grizzly Bears", Zone::Battlefield);
    let old_land = put(&mut parent, player, "Forest", Zone::Graveyard);
    let first = put(&mut parent, player, "Forest", Zone::Library);
    let nonland = put(&mut parent, player, "Hill Giant", Zone::Library);
    let second = put(&mut parent, player, "Island", Zone::Library);
    let enemy_land = put(&mut parent, opponent, "Island", Zone::Library);
    let mill = spawn(&mut parent, player, opt, Zone::Hand);
    let exile = spawn(&mut parent, opponent, growth, Zone::Hand);

    cast(&mut parent, mill, None);
    pass_twice(&mut parent);
    let first_grave = parent.object_ref(first);
    let second_grave = parent.object_ref(second);
    for viewer in [player, opponent] {
        let view = parent.observe(viewer);
        assert_eq!(view.stack.len(), 1);
        let ability = &view.stack[0];
        assert_eq!(
            ability.source().unwrap().object,
            parent.object_ref(observer)
        );
        assert!(ability.targets.is_empty());
        let arrivals = ability.event_object.as_deref().unwrap();
        assert_eq!(arrivals.object, second_grave);
        assert_eq!(
            arrivals.semantic_objects.as_deref().unwrap().as_slice(),
            &[DamageSourceRef::Object(first_grave)]
        );
    }
    act(&mut parent, Action::Pass);
    let revision = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == Action::Pass)
        .unwrap();
    let checkpoint = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    let mut stale = parent.clone();
    drop(parent);
    act(&mut raw, Action::Pass);
    indexed.apply_index(opponent, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    for land in [first, second] {
        let object = raw.object_view(land);
        assert_eq!(object.zone, Zone::Battlefield);
        assert!(object.tapped);
        assert_eq!(object.controller, player);
    }
    for card in [old_land, nonland, enemy_land] {
        assert_eq!(raw.object_view(card).zone, Zone::Graveyard);
    }
    indexed.restore(&checkpoint).unwrap();
    indexed
        .apply_index(opponent, indexed.revision(), index)
        .unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());

    cast(&mut stale, exile, Some(Target::Card(first)));
    pass_twice(&mut stale);
    assert_eq!(stale.object_view(first).zone, Zone::Exile);
    stale.move_object(first, Zone::Graveyard);
    assert_ne!(stale.object_ref(first), first_grave);
    assert!(stale.state.pending_triggers.is_empty());
    settle(&mut stale);
    assert_eq!(stale.object_view(first).zone, Zone::Graveyard);
    assert_eq!(stale.object_view(second).zone, Zone::Battlefield);
    assert!(stale.object_view(second).tapped);
}

#[test]
fn cards_redirected_to_exile_do_not_create_a_graveyard_arrival() {
    let opt = Card::from_name("Opt").unwrap();
    let _rules =
        fixture_rules::activate(&[(Card::GrizzlyBears, &LAND_RECOVERY), (opt, &MILL_BOTH)]);
    let player = Player::P1;
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Rest in Peace",
        Zone::Battlefield,
    );
    let land = put(&mut game, player, "Island", Zone::Library);
    let mill = spawn(&mut game, player, opt, Zone::Hand);
    cast(&mut game, mill, None);
    pass_twice(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    assert!(game.observe(player).stack.is_empty());
    assert_eq!(
        game.observe(player).players[player.index()].lands_entered_this_turn,
        0
    );
}

fn announce_shuffle(game: &mut Game, spell: CardId, permanent: CardId, grave_cards: &[CardId]) {
    act(game, Action::AnnounceCast { card: spell });
    for &card in std::iter::once(&permanent).chain(grave_cards) {
        act(
            game,
            Action::ChooseTarget {
                target: Target::Card(card),
            },
        );
    }
    act(game, Action::FinishTargets);
    pay(game);
    pass_twice(game);
    assert_eq!(
        game.legal_actions(),
        vec![
            Action::ChooseOptional { accept: true },
            Action::ChooseOptional { accept: false },
        ]
    );
}

#[test]
fn optional_shuffle_uses_the_permanents_owner_and_moves_the_graveyard_group_once() {
    let growth = Card::from_name("Giant Growth").unwrap();
    let _rules = fixture_rules::activate(&[
        (growth, &SHUFFLE),
        (Card::GrizzlyBears, &GRAVEYARD_DEPARTURE),
    ]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let permanent = put(&mut game, player, "Fountainport Bell", Zone::Battlefield);
    game.change_controller(game.object_ref(permanent), opponent);
    assert_eq!(game.object_view(permanent).controller, opponent);
    let first = put(&mut game, player, "Island", Zone::Graveyard);
    let second = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let unselected = put(&mut game, player, "Forest", Zone::Graveyard);
    let spell = spawn(&mut game, player, growth, Zone::Hand);
    let foreign_library = game.state.players[opponent.index()].library.clone();

    announce_shuffle(&mut game, spell, permanent, &[first, second]);
    assert!(
        game.state.players[player.index()]
            .library
            .contains(&permanent)
    );
    assert_eq!(
        game.state.players[opponent.index()].library,
        foreign_library
    );
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);
    assert_eq!(game.object_view(second).zone, Zone::Graveyard);

    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    settle(&mut declined);
    assert_eq!(declined.object_view(first).zone, Zone::Graveyard);
    assert_eq!(declined.object_view(second).zone, Zone::Graveyard);
    assert_eq!(
        declined.observe(player).players[player.index()].cards_drawn_this_turn,
        0
    );

    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.observe(player).stack.len(), 1);
    for card in [first, second] {
        assert_eq!(game.object_view(card).zone, Zone::Library);
    }
    assert_eq!(game.object_view(unselected).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(
        game.observe(player).players[player.index()].cards_drawn_this_turn,
        1
    );
}

#[test]
fn accepting_a_shuffle_with_zero_graveyard_targets_shuffles_the_library() {
    let growth = Card::from_name("Giant Growth").unwrap();
    let _rules = fixture_rules::activate(&[(growth, &SHUFFLE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let top = put(&mut game, player, "Island", Zone::Library);
    game.remember_library_top(player, game.object_ref(top));
    let permanent = put(&mut game, opponent, "Fountainport Bell", Zone::Battlefield);
    let spell = spawn(&mut game, player, growth, Zone::Hand);

    announce_shuffle(&mut game, spell, permanent, &[]);
    assert!(game.observe(opponent).known_library_top[player.index()].is_some());
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert!(declined.observe(opponent).known_library_top[player.index()].is_some());

    let mut expected = game.state.players[player.index()].library.to_vec();
    let mut rng = game.state.rng;
    rng.shuffle(&mut expected);
    act(&mut game, Action::ChooseOptional { accept: true });
    assert!(game.observe(opponent).known_library_top[player.index()].is_none());
    assert_eq!(&game.state.players[player.index()].library[..], &expected);
    assert_eq!(game.state.rng, rng);
}
