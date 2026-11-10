//! Dynamic group unions and exact predeparture quantities and observers.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::engine::characteristic_query::CharacteristicQuery;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::{cards::*, engine::*};
use support::*;

const ARTIFICERS: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Artificer"),
    basic: false,
    other: false,
    cards_only: false,
};
const CREATURES: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};
const ARTIFICER_BONUS: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::CreatureGroupScaling {
            filters: &[
                CreatureFilter {
                    artifact: true,
                    ..CREATURES
                },
                CreatureFilter {
                    subtype: Some("Hero"),
                    ..CREATURES
                },
            ],
            value: StatValue::Sum(&[
                StatValue::Count(ARTIFICERS),
                StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    cards_only: true,
                    ..ARTIFICERS
                }),
            ]),
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
    }],
    ..CardRules::EMPTY
};

const OBSERVER: CardRules = CardRules {
    effect: Effect::Land,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfOrControlledPermanentDies(1 | 16),
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

const ALL_PERMANENTS: BattlefieldFilter = BattlefieldFilter {
    kinds: 0,
    relation: -1,
    nonland: false,
    tapped: None,
    without: Keywords::NONE,
    subtype: None,
    max_mana: u32::MAX,
};

const SWEEP: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::MoveBattlefield {
            filters: &[ALL_PERMANENTS],
            exile: false,
        }],
    ),
    ..CardRules::EMPTY
};

const COUNTER_LEGENDS: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::PutLegendaryPermanentCountersScaled {
            filters: &[BattlefieldFilter {
                kinds: 1,
                relation: 0,
                ..ALL_PERMANENTS
            }],
            kind: CounterKind::PlusOne,
            amount: AmountValue::SourceCounterQuantity(CounterKind::PlusOne),
        }],
    }],
    ..CardRules::EMPTY
};

#[test]
fn creature_group_bonus_counts_overlap_once_in_actual_and_projected_worlds() {
    let cid = Card::from_name("Cid, Timeless Artificer").unwrap();
    let _rules = fixture_rules::activate(&[(cid, &ARTIFICER_BONUS)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let provider = spawn(&mut game, player, cid, Zone::Battlefield);
    let graveyard = spawn(&mut game, player, cid, Zone::Graveyard);
    let artifact = put(&mut game, player, "Magitek Infantry", Zone::Battlefield);
    let noncreature = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let ordinary = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let foreign = put(
        &mut game,
        player.other(),
        "Magitek Infantry",
        Zone::Battlefield,
    );
    let hero = game.state.objects.len() as CardId;
    game.create_tokens(player, Card(5328), 1, false);
    let dual = game.state.objects.len() as CardId;
    game.stage_copy_creation(
        player,
        CopyRecipe::new(Card(5328)).with_exceptions(CopyExceptions {
            artifact: true,
            ..CopyExceptions::NONE
        }),
        1,
    );
    assert!(!game.advance_entries());
    assert_eq!(game.object_view(dual).kinds & (1 | 16), 1 | 16);

    assert_eq!(stats(&game, artifact), (4, 3));
    for recipient in [hero, dual] {
        assert_eq!(stats(&game, recipient), (3, 3));
    }
    assert_eq!(stats(&game, ordinary), (2, 2));
    assert_eq!(stats(&game, foreign), (1, 1));
    assert_eq!(stats(&game, noncreature), (0, 0));

    let checkpoint = game.snapshot();
    let goal = FundingGoal {
        actor: player,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    };
    let projected = FundingState::from_game(&game, &goal);
    assert_eq!(
        CharacteristicQuery::in_world(&projected).stats(dual).power,
        3
    );
    assert!(projected.cost_departures(
        &[game.object_ref(provider)],
        Zone::Graveyard,
        &mut |world| {
            assert_eq!(CharacteristicQuery::in_world(world).stats(dual).power, 1);
            true
        },
    ));
    assert_eq!(game.snapshot(), checkpoint);

    game.move_object(graveyard, Zone::Hand);
    assert_eq!(stats(&game, dual), (2, 2));
    game.record_control(game.object_ref(dual), player.other(), None);
    assert_eq!(stats(&game, dual), (1, 1));
}

#[test]
fn exact_self_death_and_other_controlled_deaths_use_one_packet_per_event() {
    let sweep = Card::from_name("Day of Judgment").unwrap();
    let _rules = fixture_rules::activate(&[(Card::Forest, &OBSERVER), (sweep, &SWEEP)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, Card::Forest, Zone::Battlefield);
    put(&mut game, player, "Plains", Zone::Battlefield);
    put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    put(&mut game, player, "Magitek Infantry", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let removed = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.move_object(removed, Zone::Exile);
    assert!(game.state.pending_triggers.is_empty());
    assert!(game.state.stack.is_empty());
    assert_eq!(game.object_view(source).kinds & (1 | 16), 0);
    let spell = spawn(&mut game, player, sweep, Zone::Hand);
    cast(&mut game, spell, None);
    act(&mut game, Action::Pass);
    let mut indexed = game.clone();
    let pass_index = game
        .legal_actions()
        .iter()
        .position(|a| *a == Action::Pass)
        .unwrap();
    let revision = game.revision();
    act(&mut game, Action::Pass);
    indexed
        .apply_index(player.other(), revision, pass_index)
        .unwrap();
    assert_eq!(indexed.snapshot(), game.snapshot());
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    let packets = game
        .state
        .stack
        .iter()
        .chain(&game.state.pending_triggers)
        .chain(&game.state.ordering_triggers)
        .filter(|item| item.source.is_some_and(|row| row.id == source))
        .count();
    assert_eq!(packets, 3);

    for _ in 0..24 {
        if game.state.players[player.index()].life == 23 {
            break;
        }
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. } | Action::Pass)
        });
    }
    assert_eq!(game.state.players[player.index()].life, 23);
}

#[test]
fn legendary_recipients_are_live_and_receive_only_frozen_plus_one_counters() {
    let aerith = Card::from_name("Aerith Gainsborough").unwrap();
    let _rules = fixture_rules::activate(&[(aerith, &COUNTER_LEGENDS)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = spawn(&mut game, player, aerith, Zone::Battlefield);
    let legend = put(&mut game, player, "Adelbert Steiner", Zone::Battlefield);
    let plain = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let foreign = put(
        &mut game,
        player.other(),
        "Minwu, White Mage",
        Zone::Battlefield,
    );
    game.state.objects[source as usize].counters.plus_one = 2;
    game.state.objects[source as usize].counters.stun = 1;
    game.move_object(source, Zone::Graveyard);
    game.prepare_triggers();
    assert_eq!(game.state.stack.len(), 1);
    game.move_object(source, Zone::Battlefield);
    game.state.objects[source as usize].counters.plus_one = 1;
    settle(&mut game);
    assert_eq!(game.object_view(legend).counters.plus_one, 2);
    assert_eq!(game.object_view(source).counters.plus_one, 3);
    for excluded in [plain, foreign] {
        assert_eq!(game.object_view(excluded).counters.plus_one, 0);
    }
}
