//! An optional exact source sacrifice has its own target announcement.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const YOUR_GRAVEYARD: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const YOUR_PERMANENT_CARD: TargetRule = TargetRule::Filtered {
    base: &YOUR_GRAVEYARD,
    predicate: &TargetPredicate::Permanent,
};
const ANOTHER_PERMANENT: TargetRule = TargetRule::Filtered {
    base: &YOUR_PERMANENT_CARD,
    predicate: &TargetPredicate::OtherThanPhysicalSource,
};
const COLORLESS: Mana = [0, 0, 0, 0, 0, 1];
const EDEN: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed(COLORLESS))],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                generic: 5,
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(2)),
                Instruction::MaySacrificeSourceReflexive { ability: 0 },
            ],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::Single(TargetSpec::one(ANOTHER_PERMANENT)),
        program: &[Instruction::ReturnToHand],
    }],
    ..CardRules::EMPTY
};

fn eden() -> Card {
    Card::from_name("Eden, Seat of the Sanctum").unwrap()
}

fn announce_sacrifice(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 1,
        counter_payment: Counters::default(),
    }
}

fn begin(game: &mut Game, player: Player, source: CardId) {
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 5];
    act(game, announce_sacrifice(source));
    pay(game);
    assert!(game.object_view(source).tapped);
    assert!(game.state.stack[0].targets.is_empty());
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn choose_return(game: &mut Game, card: CardId) {
    act(game, target(card));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

#[test]
fn the_post_sacrifice_trigger_excludes_its_physical_card_and_can_return_a_namesake() {
    let _rules = fixture_rules::activate(&[(eden(), &EDEN)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, eden(), Zone::Battlefield);
    let namesake = spawn(&mut game, player, eden(), Zone::Graveyard);
    let foreign = put(
        &mut game,
        player.other(),
        "Swiftfoot Boots",
        Zone::Graveyard,
    );
    let instant = put(&mut game, player, "Shock", Zone::Library);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let exact = game.object_ref(source);

    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 4];
    reject(&mut game, announce_sacrifice(source)); // The source tap is reserved.
    let mut mana = game.clone();
    act(
        &mut mana,
        Action::ActivateMana {
            card: source,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: COLORLESS,
        },
    );
    assert!(mana.state.stack.is_empty());
    assert_eq!(mana.state.players[player.index()].mana[5], 5);
    reject(&mut mana, announce_sacrifice(source)); // The source is already tapped.

    begin(&mut game, player, source);
    let activation = game.state.stack[0].id;
    assert_eq!(game.object_view(creature).zone, Zone::Library);
    pass_pair(&mut game);
    assert!(matches!(game.state.pending, Pending::EffectOptional { .. }));
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    reject(&mut game, target(creature));

    let accept = Action::ChooseOptional { accept: true };
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == accept)
        .unwrap();
    let revision = game.revision();
    let saved = game.snapshot();
    let mut raw = game.clone();
    act(&mut raw, accept);
    game.apply_index(player, revision, index).unwrap();
    for branch in [&mut raw, &mut game] {
        assert_eq!(branch.state.pending, Pending::Targets { player });
        assert_eq!(branch.object_view(source).zone, Zone::Graveyard);
        assert_ne!(branch.object_ref(source), exact);
        for illegal in [source, foreign, instant] {
            reject(branch, target(illegal));
        }
        assert!(branch.legal_actions().contains(&target(creature)));
        choose_return(branch, namesake);
        let trigger = branch.state.stack.last().unwrap();
        assert_ne!(trigger.id, activation);
        assert_eq!(trigger.source().unwrap().object, exact);
        assert!(matches!(trigger.kind, StackKind::TriggeredAbility { .. }));
        assert_eq!(branch.object_view(namesake).zone, Zone::Graveyard);
        pass_pair(branch);
        assert_eq!(branch.object_view(namesake).zone, Zone::Hand);
        assert_eq!(branch.object_view(source).zone, Zone::Graveyard);
    }
    assert_eq!(raw.snapshot(), game.snapshot());

    raw.restore(&saved).unwrap();
    act(&mut raw, Action::ChooseOptional { accept: true });
    choose_return(&mut raw, creature);
    let target = raw.object_ref(creature);
    raw.move_object(creature, Zone::Exile);
    raw.move_object(creature, Zone::Graveyard);
    assert_ne!(raw.object_ref(creature), target);
    settle(&mut raw);
    assert_eq!(raw.object_view(creature).zone, Zone::Graveyard);
}

#[test]
fn milling_commits_before_decline_or_a_successful_sacrifice_with_no_legal_target() {
    let _rules = fixture_rules::activate(&[(eden(), &EDEN)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, eden(), Zone::Battlefield);
    let first = put(&mut game, player, "Shock", Zone::Library);
    let second = put(&mut game, player, "Unsummon", Zone::Library);
    begin(&mut game, player, source);
    pass_pair(&mut game);
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    act(&mut game, Action::ChooseOptional { accept: true });
    for branch in [&decline, &game] {
        assert_eq!(branch.object_view(first).zone, Zone::Graveyard);
        assert_eq!(branch.object_view(second).zone, Zone::Graveyard);
        assert_eq!(branch.state.pending, Pending::Priority);
        assert!(branch.state.stack.is_empty());
    }
    assert_eq!(decline.object_view(source).zone, Zone::Battlefield);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
}

#[test]
fn stale_stolen_and_sacrifice_prohibited_sources_still_mill_and_offer_no_payment() {
    let _rules = fixture_rules::activate(&[(eden(), &EDEN)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, eden(), Zone::Battlefield);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Library);
    let exact = game.object_ref(source);
    begin(&mut game, player, source);
    let mut stale = game.clone();
    stale.move_object(source, Zone::Exile);
    stale.move_object(source, Zone::Battlefield);
    assert_ne!(stale.object_ref(source), exact);
    let mut stolen = game.clone();
    stolen.record_control(exact, player.other(), None);
    assert_eq!(stolen.object_view(source).controller, player.other());
    game.state.objects[source as usize].gained_keywords = Keywords::CANT_BE_SACRIFICED;
    for branch in [&mut stale, &mut stolen, &mut game] {
        pass_pair(branch);
        assert_eq!(branch.state.pending, Pending::Priority);
        assert!(branch.state.stack.is_empty());
        assert_eq!(branch.object_view(source).zone, Zone::Battlefield);
        assert_eq!(branch.object_view(creature).zone, Zone::Graveyard);
        reject(branch, Action::ChooseOptional { accept: true });
    }
}

#[test]
fn a_replaced_sacrifice_still_triggers_but_a_prevented_move_does_not() {
    struct ResetPrevention;
    impl Drop for ResetPrevention {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
        }
    }

    let _rules = fixture_rules::activate(&[(eden(), &EDEN)]);
    let _reset = ResetPrevention;
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, eden(), Zone::Battlefield);
    let target = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let rest = put(
        &mut game,
        player.other(),
        "Rest in Peace",
        Zone::Battlefield,
    );
    begin(&mut game, player, source);
    pass_pair(&mut game);
    let mut blocked = game.clone();
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.object_view(source).zone, Zone::Exile);
    choose_return(&mut game, target);
    settle(&mut game);
    assert_eq!(game.object_view(target).zone, Zone::Hand);

    let exact = blocked.object_ref(source);
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(exact)));
    // Remove the competing exile replacement to exercise the prevented move.
    blocked.move_object(rest, Zone::Exile);
    act(&mut blocked, Action::ChooseOptional { accept: true });
    assert_eq!(blocked.state.pending, Pending::Priority);
    assert!(blocked.state.stack.is_empty());
    assert_eq!(blocked.object_ref(source), exact);
    assert_eq!(blocked.object_view(target).zone, Zone::Graveyard);
}
