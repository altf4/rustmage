//! Class levels change live ward grants and actor-scoped counter replacement.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::characteristic_query::CharacteristicQuery;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::replacements::ResumeMode;
use rustmage_core::engine::*;
use support::*;

const DOUBLE: ReplacementSpec = ReplacementSpec::DoubleCountersPlacedByController {
    condition: StatValue::SourceClassLevelAtLeast(3),
};
const CLASS: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::Ward),
        targets: TargetPlan::NONE,
        program: &[Instruction::CounterUnlessPay {
            cost: ManaCost {
                colored: [0; 6],
                generic: 1,
            },
            subject: CounterSubject::Related,
            exile: false,
            unpaid_skip: 0,
        }],
    }],
    activated: &[
        CLASS_LEVEL_FIELDS,
        ActivatedAbility {
            guard: ActivationGuard::ClassLevelExactly(2),
            cost: ActivationCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 3,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::GainClassLevel(3)],
            ..CLASS_LEVEL_FIELDS
        },
    ],
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::CounterBearingPermanentWard {
                condition: StatValue::SourceClassLevelAtLeast(2),
                ability: 0,
            }],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Replacement(0, DOUBLE)],
        },
    ],
    extras: &CardExtras {
        replacement_effects: &[DOUBLE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const CLASS_LEVEL_FIELDS: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::ClassLevelExactly(1),
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        colored: [0, 0, 0, 0, 1, 0],
        mana_component: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::GainClassLevel(2)],
    sorcery: true,
    once_per_turn: false,
};
const MIXED_PLACEMENT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::PutCountersMixed {
            subject: Subject::Targets,
            amounts: &[
                (CounterKind::PlusOne, 1),
                (CounterKind::Charge, 2),
                (CounterKind::Bait, 0),
            ],
        }],
    ),
    ..CardRules::EMPTY
};
const ENTERS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    enters: InitialCounters {
        plus_one: 1,
        ..CardRules::EMPTY.enters
    },
    ..CardRules::EMPTY
};
fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn class(g: &mut Game, p: Player, level: u8) -> CardId {
    let card = put(g, p, "Hunter's Talent", Zone::Battlefield);
    g.state.objects[card as usize].class_level = level;
    card
}
fn level_up(g: &mut Game, source: CardId, slot: u16) {
    choose(
        g,
        |action| matches!(action, Action::AnnounceActivate { card, ability, .. } if *card == source && *ability == slot),
    );
    pay(g);
    settle(g);
}
fn suppress(g: &mut Game, source: CardId) {
    let reference = g.object_ref(source);
    let controller = g.object_view(source).controller;
    g.remove_abilities_exact(
        &[reference],
        reference,
        controller,
        AnimationDuration::Permanent,
    );
}
fn priority(g: &mut Game, player: Player) {
    if g.actor() != Some(player) {
        act(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(player));
}
fn ward_count(g: &Game, recipient: CardId) -> usize {
    g.characteristic_query().static_ward_grants(recipient).len()
}

#[test]
fn ward_begins_at_level_two_and_lives_on_noncreatures_with_named_counters() {
    let _rules = fixture_rules::activate(&[(named("Hunter's Talent"), &CLASS)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let primary = class(&mut g, p, 1);
        let secondary = class(&mut g, p, 2);
        suppress(&mut g, secondary);
        let artifact = put(&mut g, p, "Short Bow", Zone::Battlefield);
        let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let empty = put(&mut g, p, "Short Bow", Zone::Battlefield);
        let foreign = put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
        g.add_counter(artifact, CounterKind::Charge, 1);
        g.add_counter(creature, CounterKind::PlusOne, 1);
        g.add_counter(foreign, CounterKind::Charge, 1);
        assert_eq!(ward_count(&g, artifact), 0);
        level_up(&mut g, primary, 0);
        assert_eq!(g.object_view(primary).class_level, Some(2));
        assert_eq!(ward_count(&g, artifact), 1);
        assert_eq!(ward_count(&g, creature), 1);
        assert_eq!(ward_count(&g, empty), 0);
        assert_eq!(ward_count(&g, foreign), 0);

        let third = class(&mut g, p, 2);
        assert_eq!(ward_count(&g, artifact), 2);
        suppress(&mut g, third);
        level_up(&mut g, primary, 1);
        assert_eq!(g.object_view(primary).class_level, Some(3));

        priority(&mut g, p.other());
        let disenchant = put(&mut g, p.other(), "Disenchant", Zone::Hand);
        cast(&mut g, disenchant, Some(Target::Card(artifact)));
        g.set_counters(artifact, Counters::default());
        suppress(&mut g, primary);
        assert_eq!(ward_count(&g, artifact), 0);
        for _ in 0..8 {
            if g.legal_actions().contains(&Action::DeclineResolutionMana) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        let mut paid = g.clone();
        choose(
            &mut paid,
            |a| matches!(a, Action::PayResolutionMana { payment } if payment.iter().sum::<u32>() == 1),
        );
        settle(&mut paid);
        assert_eq!(paid.object_view(artifact).zone, Zone::Graveyard);
        act(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
    }
}

#[test]
fn permanent_counter_replacement_uses_placer_for_every_positive_kind() {
    let _rules = fixture_rules::activate(&[
        (named("Hunter's Talent"), &CLASS),
        (named("Murder"), &MIXED_PLACEMENT),
    ]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = class(&mut g, p, 2);
        let muted = class(&mut g, p, 3);
        suppress(&mut g, muted);
        let yours = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let theirs = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);

        let spell = put(&mut g, p, "Murder", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(theirs)));
        settle(&mut g);
        assert_eq!(g.counter_count(theirs, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(theirs, CounterKind::Charge), 2);

        level_up(&mut g, provider, 1);
        g.state.players[p.index()].mana = [8; 6];
        let spell = put(&mut g, p, "Murder", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(theirs)));
        settle(&mut g);
        assert_eq!(g.counter_count(theirs, CounterKind::PlusOne), 3);
        assert_eq!(g.counter_count(theirs, CounterKind::Charge), 6);
        assert_eq!(g.counter_count(theirs, CounterKind::Bait), 0);

        priority(&mut g, p.other());
        let spell = put(&mut g, p.other(), "Murder", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(yours)));
        settle(&mut g);
        assert_eq!(g.counter_count(yours, CounterKind::PlusOne), 1);
        assert_eq!(g.counter_count(yours, CounterKind::Charge), 2);
    }
}

#[test]
fn player_counter_replacement_choices_are_owned_by_the_affected_seat() {
    let _rules = fixture_rules::activate(&[(named("Hunter's Talent"), &CLASS)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let first = class(&mut g, p, 3);
        class(&mut g, p, 3);
        class(&mut g, p.other(), 3);
        let mut amounts = [[0; 5]; 2];
        amounts[p.other().index()][PlayerCounterKind::Energy.index()] = 2;
        amounts[p.other().index()][PlayerCounterKind::Poison.index()] = 1;
        g.effect_counters_with_players(Vec::new(), amounts, p, ResumeMode::Pending);
        assert!(g.advance_replacements(false));
        assert_eq!(g.actor(), Some(p.other()));
        let action = Action::ChooseReplacement {
            source: g.object_ref(first),
            ability: 0,
        };
        let index = g.legal_actions().iter().position(|a| a == &action).unwrap();
        let revision = g.revision();
        let checkpoint = g.snapshot();
        let mut indexed = g.clone();
        assert_eq!(g.apply(p, action.clone()), Err(Error::WrongActor));
        assert_eq!(g.snapshot(), checkpoint);
        act(&mut g, action.clone());
        indexed.apply_index(p.other(), revision, index).unwrap();
        assert_eq!(g.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(indexed.snapshot(), checkpoint);
        assert_eq!(
            indexed.apply_index(p.other(), revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, action.clone());
        for branch in [&mut g, &mut indexed] {
            act(branch, action.clone());
            assert_eq!(branch.state.pending, Pending::Priority);
            let recipient = &branch.state.players[p.other().index()];
            assert_eq!(recipient.player_counter(PlayerCounterKind::Energy), 8);
            assert_eq!(recipient.player_counter(PlayerCounterKind::Poison), 4);
            assert_eq!(recipient.player_counter(PlayerCounterKind::Ticket), 0);
            assert_eq!(
                branch.state.players[p.index()].player_counter(PlayerCounterKind::Poison),
                0
            );
        }
        assert_eq!(g.snapshot(), indexed.snapshot());
    }
}

#[test]
fn entry_counters_and_granted_ward_agree_in_projected_and_live_worlds() {
    let _rules = fixture_rules::activate(&[
        (named("Hunter's Talent"), &CLASS),
        (named("Bear Cub"), &ENTERS),
    ]);
    for active in [Player::P0, Player::P1] {
        for entrant_controller in [active, active.other()] {
            let mut g = scenario(active);
            let provider = class(&mut g, active, 3);
            let entrant = put(
                &mut g,
                entrant_controller.other(),
                "Bear Cub",
                Zone::Graveyard,
            );
            let mut movement = g.entry_move(entrant, Zone::Battlefield, false);
            movement.controller = entrant_controller;
            let goal = FundingGoal {
                actor: entrant_controller,
                required: [0; 6],
                generic: 0,
                reserve: None,
                source: None,
            };
            let world = FundingState::from_game(&g, &goal);
            let expected = if entrant_controller == active {
                (2, 1)
            } else {
                (1, 0)
            };
            let mut outcomes = Vec::new();
            world.visit_entry_moves(vec![movement.clone()], true, None, &mut |next| {
                let query = CharacteristicQuery::in_world(next);
                let grants = query.static_ward_grants(entrant);
                assert!(grants.iter().all(|grant| grant.grantor.card == provider));
                outcomes.push((
                    CharacteristicWorld::counter_count(next, entrant, CounterKind::PlusOne),
                    grants.len(),
                ));
                false
            });
            assert_eq!(outcomes, vec![expected]);

            g.stage_entries(vec![movement], None);
            assert!(!g.advance_entries());
            assert_eq!(g.object_view(entrant).controller, entrant_controller);
            assert_eq!(
                (
                    g.counter_count(entrant, CounterKind::PlusOne),
                    ward_count(&g, entrant)
                ),
                expected
            );
        }
    }
}
