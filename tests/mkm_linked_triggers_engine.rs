//! Exact linked copies and triggering cast or sacrifice objects.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const CAST_KEYWORDS: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::InstantSorcery,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Give that spell deathtouch and lifelink.",
                targets: TargetPlan::NONE,
                program: &[Instruction::GrantCastSpellKeywords(
                    Keywords::DEATHTOUCH.union(Keywords::LIFELINK),
                )],
            },
            SpellMode {
                text: "Gain 1 life.",
                targets: TargetPlan::NONE,
                program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
            },
        ])],
    }],
    ..CardRules::EMPTY
};

#[test]
fn granting_lifelink_to_the_cast_spell_uses_its_actual_damage_dealer() {
    let creature = Card::from_name("Hill Giant").unwrap();
    let _rules = fixture_rules::activate(&[(creature, &CAST_KEYWORDS)]);
    let player = Player::P0;
    let mut game = scenario(player);
    spawn(&mut game, player, creature, Zone::Battlefield);
    let spell = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, spell, Some(Target::Player(player.other())));
    let mut alternate = game.clone();
    act(&mut alternate, Action::ChooseMode { mode: 1 });
    settle(&mut alternate);
    assert_eq!(alternate.state.players[player.index()].life, 21);
    act(&mut game, Action::ChooseMode { mode: 0 });
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 22);
    assert_eq!(game.state.players[player.other().index()].life, 18);

    let large = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(large)));
    act(&mut game, Action::ChooseMode { mode: 0 });
    settle(&mut game);
    assert_eq!(game.object_view(large).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 24);
}

const SACRIFICE_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InZone {
            zone: TargetZone::Graveyard,
            event: &TriggerCondition::Sacrificed {
                own_turn: false,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: Some("Equipment"),
                },
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnSourceToHand(TargetZone::Graveyard)],
    }],
    ..CardRules::EMPTY
};

#[test]
fn the_graveyard_observer_follows_actual_own_sacrifices_and_exact_identity() {
    let creature = Card::from_name("Hill Giant").unwrap();
    let _rules = fixture_rules::activate(&[(creature, &SACRIFICE_OBSERVER)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, creature, Zone::Graveyard);
    let opposing = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    let own = put(&mut game, player, "Short Bow", Zone::Battlefield);
    game.sacrifice_batch(&[(player.other(), opposing)]);
    assert!(game.state.pending_triggers.is_empty());
    game.sacrifice_batch(&[(player, own)]);
    game.prepare_triggers();
    let mut stale = game.clone();
    stale.move_object(source, Zone::Exile);
    stale.move_object(source, Zone::Graveyard);
    settle(&mut stale);
    assert_eq!(stale.object_view(source).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Hand);
}

const EXILE_ALL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Creature),
            TargetSpec::one(TargetRule::Creature),
        ]),
        &[Instruction::ExileAllTargets],
    ),
    ..CardRules::EMPTY
};

#[test]
fn exile_across_independent_target_groups_moves_an_exact_card_only_once() {
    let spell = Card::from_name("Opt").unwrap();
    let _rules = fixture_rules::activate(&[(spell, &EXILE_ALL)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let card = spawn(&mut game, player, spell, Zone::Hand);
    let creature = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let before = game.object_ref(creature);
    announce(
        &mut game,
        card,
        &[Target::Card(creature), Target::Card(creature)],
    );
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Exile);
    assert_eq!(game.object_ref(creature).generation, before.generation + 1);
}

const COPY_LINKED: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayCopyLinkedExiledCreatureUntilEndOfTurn,
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};

#[test]
fn linked_copy_is_exact_optional_and_lasts_through_this_turn() {
    let creature = Card::from_name("Hill Giant").unwrap();
    let _rules = fixture_rules::activate(&[(creature, &COPY_LINKED)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, creature, Zone::Battlefield);
    let exact = game.object_ref(source);
    let donor = put(&mut game, player.other(), "Llanowar Elves", Zone::Exile);
    let wrong = put(&mut game, player.other(), "Forest", Zone::Exile);
    let donor = game.object_ref(donor);
    let wrong = game.object_ref(wrong);
    for exiled in [donor, wrong] {
        game.link_exile_to_ability(exact, None, exiled);
    }
    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(&mut game);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    reject(&mut game, Action::ChooseLinkedExile { object: wrong });
    reject(&mut game, Action::ChooseOptional { accept: true });
    let decline = Action::ChooseOptional { accept: false };
    let decline_index = game
        .legal_actions()
        .iter()
        .position(|action| *action == decline)
        .unwrap();
    let mut declined = game.clone();
    let mut indexed = game.clone();
    act(&mut declined, decline);
    indexed
        .apply_index(player, game.revision(), decline_index)
        .unwrap();
    assert_eq!(declined.snapshot(), indexed.snapshot());
    assert_eq!(declined.object_view(source).card, creature);
    assert_eq!(declined.state.players[player.index()].life, 21);
    assert_eq!(declined.object_ref(donor.card), donor);
    act(&mut game, Action::ChooseLinkedExile { object: donor });
    assert_eq!(game.state.players[player.index()].life, 21);
    assert_eq!(game.object_ref(source), exact);
    assert_eq!(
        game.object_view(source).card,
        Card::from_name("Llanowar Elves").unwrap()
    );
    game.state.step = Step::End;
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.object_view(source).card, creature);
}
