//! Named recipient counters use current characteristics and exact death power.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const NAME: &str = "Grizzly Bears";
const TRIPLETS: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::TRAMPLE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfNontokenEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateCopyToken {
                subject: Subject::Source,
                amount: 2,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::PutNamedPermanentCountersScaled {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                name: NAME,
                kind: CounterKind::PlusOne,
                amount: AmountValue::SourcePower,
            }],
        },
    ],
    ..CardRules::EMPTY
};
const RETURN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::ReturnToBattlefield],
    ),
    ..CardRules::EMPTY
};
const WATCH: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CounterBatchPlacedByYou {
            kind: CounterKind::PlusOne,
            filter: CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 1,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn named_copy(game: &mut Game, owner: Player, definition: Card, name: &'static str) -> CardId {
    let id = game.state.objects.len() as CardId;
    let recipe = CopyRecipe {
        overrides: Some(CopyOverrides {
            name: Some(name),
            ..CopyOverrides::NONE
        }),
        ..CopyRecipe::new(definition)
    };
    game.stage_copy_creation(owner, recipe, 1);
    assert!(!game.advance_entries());
    assert_eq!(game.object_view(id).name(), name);
    id
}

fn kill_until_trigger(game: &mut Game, murder: CardId, source: CardId) {
    let departed = game.object_ref(source);
    cast(game, murder, Some(Target::Card(source)));
    for _ in 0..16 {
        if game.state.stack.last().is_some_and(|item| {
            item.kind == StackKind::TriggeredAbility { ability: 1 }
                && item.source.is_some_and(|source| source.object == departed)
        }) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("the death trigger did not reach the stack");
}

#[test]
fn death_uses_the_old_power_for_one_live_name_batch_including_copy_tokens() {
    let _rules = fixture_rules::activate(&[
        (card(NAME), &TRIPLETS),
        (card("Think Twice"), &RETURN),
        (card("Gilded Lotus"), &WATCH),
    ]);
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, NAME, Zone::Hand);
    let opposing = put(&mut game, player.other(), NAME, Zone::Battlefield);
    put(&mut game, player, "Gilded Lotus", Zone::Battlefield);
    let pump = put(&mut game, player, "Giant Growth", Zone::Hand);
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    let returned = put(&mut game, player, "Think Twice", Zone::Hand);
    let alias = named_copy(&mut game, player, card("Bear Cub"), NAME);
    let renamed = named_copy(&mut game, player, card(NAME), "Different Name");
    let artifact = named_copy(&mut game, player, card("Swiftfoot Boots"), NAME);

    let first = game.state.objects.len() as CardId;
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(game.state.objects.len() as CardId, first + 2);
    for token in [first, first + 1] {
        let view = game.object_view(token);
        assert!(view.is_token());
        assert_eq!(view.name(), NAME);
        assert!(view.keywords.contains(Keywords::TRAMPLE));
    }

    cast(&mut game, pump, Some(Target::Card(source)));
    settle(&mut game);
    assert_eq!(stats(&game, source), (6, 6));
    let old_source = game.object_ref(source);
    kill_until_trigger(&mut game, murder, source);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert_eq!(
        game.state
            .stack
            .last()
            .unwrap()
            .source_quantity_lki
            .as_ref()
            .unwrap()
            .power,
        Some(6)
    );

    // The reentered incarnation and its new token copies join the group while
    // the original death trigger is still waiting underneath the return spell.
    let reentered_tokens = game.state.objects.len() as CardId;
    cast(&mut game, returned, Some(Target::Card(source)));
    settle(&mut game);
    assert_ne!(game.object_ref(source), old_source);
    assert_eq!(game.state.objects.len() as CardId, reentered_tokens + 2);
    for recipient in [
        source,
        first,
        first + 1,
        alias,
        reentered_tokens,
        reentered_tokens + 1,
    ] {
        assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 6);
    }
    for excluded in [opposing, renamed, artifact] {
        assert_eq!(game.counter_count(excluded, CounterKind::PlusOne), 0);
    }
    assert_eq!(game.state.players[player.index()].life, 21);
}

#[test]
fn a_negative_power_death_creates_no_counter_event() {
    let _rules =
        fixture_rules::activate(&[(card(NAME), &TRIPLETS), (card("Gilded Lotus"), &WATCH)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, NAME, Zone::Battlefield);
    let token = named_copy(&mut game, player, card(NAME), NAME);
    put(&mut game, player, "Gilded Lotus", Zone::Battlefield);
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    game.state.objects[source as usize].growth.power = -4;
    assert_eq!(stats(&game, source), (-1, 3));
    kill_until_trigger(&mut game, murder, source);
    settle(&mut game);
    assert_eq!(game.counter_count(token, CounterKind::PlusOne), 0);
    assert_eq!(game.state.players[player.index()].life, 20);
}
