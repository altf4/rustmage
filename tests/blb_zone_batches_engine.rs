//! Entry batches use post-event types; departure batches use the frozen battlefield.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const LEAVE_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::BattlefieldDepartureBatch(DepartureRule {
            kinds: 1,
            relation: CastRelation::You,
            subject: DepartureSubject::Other,
            destinations: &[
                Zone::Hand,
                Zone::Library,
                Zone::Exile,
                Zone::Command,
                Zone::Stack,
            ],
            ..DepartureRule::ANY
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

const ENTER_OBSERVER: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersBatchYou {
            predicate: EntryPredicate::NoncreatureNonland,
            other: false,
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        program: &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};

const OTHER_SUBTYPE_OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersBatchYou {
            predicate: EntryPredicate::SubtypeAny(&["Bat", "Bird", "Rabbit", "Mouse"]),
            other: true,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

const EXILE_ALL: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::MoveBattlefield {
            filters: &[BattlefieldFilter {
                kinds: 0,
                relation: -1,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            exile: true,
        }],
    ),
    ..CardRules::EMPTY
};

const ANIMATE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 16,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::AnimateNoncreatureTargets(AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((2, 2)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        })],
    ),
    ..CardRules::EMPTY
};

const RETURN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            min: 1,
            max: 4,
            rule: TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 0,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
        }),
        &[Instruction::ReturnToBattlefield],
    ),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn resolve_return(game: &mut Game, player: Player, subjects: &[CardId]) {
    let spell = put(game, player, "Zombify", Zone::Hand);
    let targets: Vec<_> = subjects.iter().copied().map(Target::Card).collect();
    announce(game, spell, &targets);
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn one_predeparture_observer_sees_an_exiled_group_even_when_it_also_leaves() {
    let _rules = fixture_rules::activate(&[
        (Card::GrizzlyBears, &LEAVE_OBSERVER),
        (named("Fumigate"), &EXILE_ALL),
        (named("Opt"), &ANIMATE),
    ]);
    for player in [Player::P0, Player::P1] {
        for (spell, finality, expected_draws) in [
            ("Fumigate", false, 1),
            ("Day of Judgment", false, 0),
            ("Day of Judgment", true, 1),
        ] {
            let mut game = scenario(player);
            let source = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
            let opposing = put(
                &mut game,
                player.other(),
                "Grizzly Bears",
                Zone::Battlefield,
            );
            let stolen = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
            game.record_control(game.object_ref(stolen), player, None);
            let equipment = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
            let animation = put(&mut game, player, "Opt", Zone::Hand);
            cast(&mut game, animation, Some(Target::Card(equipment)));
            settle(&mut game);
            assert!(game.object_view(equipment).is_creature());
            if finality {
                game.add_counter(equipment, CounterKind::Finality, 1);
            }

            let sorcery = put(&mut game, player, spell, Zone::Hand);
            cast(&mut game, sorcery, None);
            act(&mut game, Action::Pass);
            let pass_index = game
                .legal_actions()
                .iter()
                .position(|a| *a == Action::Pass)
                .unwrap();
            let revision = game.revision();
            let mut indexed = game.clone();
            act(&mut game, Action::Pass);
            indexed
                .apply_index(player.other(), revision, pass_index)
                .unwrap();
            assert_eq!(indexed.snapshot(), game.snapshot());
            assert_ne!(game.object_view(source).zone, Zone::Battlefield);
            assert_ne!(game.object_view(opposing).zone, Zone::Battlefield);
            assert!(!game.object_view(equipment).is_creature());
            settle(&mut game);
            assert_eq!(
                game.state.players[player.index()].cards_drawn_this_turn,
                expected_draws
            );
            assert_eq!(
                game.state.players[player.other().index()].cards_drawn_this_turn,
                0
            );
        }
    }
}

#[test]
fn noncreature_nonland_entry_matches_one_batch_with_post_entry_observers() {
    let _rules = fixture_rules::activate(&[
        (named("Swiftfoot Boots"), &ENTER_OBSERVER),
        (named("Zombify"), &RETURN),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let own = put(&mut game, player, "Swiftfoot Boots", Zone::Graveyard);
        put(
            &mut game,
            player.other(),
            "Swiftfoot Boots",
            Zone::Battlefield,
        );
        let artifact = put(&mut game, player, "Expedition Map", Zone::Graveyard);
        let creature = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        let land = put(&mut game, player, "Forest", Zone::Graveyard);

        resolve_return(&mut game, player, &[own, artifact, creature, land]);
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        let qualifying: Vec<_> = [own, artifact]
            .into_iter()
            .map(|id| game.object_ref(id))
            .collect();
        let event = game
            .state
            .stack
            .last()
            .unwrap()
            .event_object
            .as_ref()
            .unwrap();
        assert_eq!(event.object, qualifying[0]);
        assert_eq!(
            event.semantic_objects.as_deref().map(Vec::as_slice),
            Some(&[DamageSourceRef::Object(qualifying[1])][..])
        );
        settle(&mut game);
        assert_eq!(game.counter_count(creature, CounterKind::PlusOne), 1);

        let another_creature = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        let another_land = put(&mut game, player, "Forest", Zone::Graveyard);
        resolve_return(&mut game, player, &[another_creature, another_land]);
        assert!(game.state.stack.is_empty());
        assert_eq!(game.state.pending, Pending::Priority);
        assert_eq!(
            game.counter_count(another_creature, CounterKind::PlusOne),
            0
        );
    }
}

#[test]
fn other_subtype_entry_counts_each_batch_once_and_excludes_the_exact_source() {
    let _rules = fixture_rules::activate(&[
        (named("Lifecreed Duo"), &OTHER_SUBTYPE_OBSERVER),
        (named("Zombify"), &RETURN),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Lifecreed Duo", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);

    let dual = put(&mut game, player, "Lifecreed Duo", Zone::Graveyard);
    let mouse = put(&mut game, player, "Nettle Guard", Zone::Graveyard);
    let nonmember = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    resolve_return(&mut game, player, &[dual, mouse, nonmember]);
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    let primary: Vec<_> = game
        .state
        .stack
        .iter()
        .map(|item| {
            (
                item.source.unwrap().id,
                item.event_object.as_ref().unwrap().object,
            )
        })
        .collect();
    assert_eq!(primary.len(), 2);
    assert!(primary.contains(&(source, game.object_ref(dual))));
    assert!(primary.contains(&(dual, game.object_ref(mouse))));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}
