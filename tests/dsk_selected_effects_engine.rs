//! Resolution-time choices among targets and actual milled/available cards.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const P: Player = Player::P0;
const ENEMY: Player = Player::P1;

const CREATURE_FILTER: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 1,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const DAMAGE_CHOICE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 2,
            max: 2,
        }),
        &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::DamageAmongTargetGroup {
                    group: 0,
                    amount: 5,
                },
                players: PlayerSet::Opponent,
                filter: CREATURE_FILTER,
                amount: 1,
                optional_skip: None,
            },
            Instruction::GrantKeywordsToUnchosenTargets {
                group: 0,
                keywords: Keywords::CANT_BLOCK,
            },
        ],
    ),
    ..CardRules::EMPTY
};

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn the_opponent_chooses_which_exact_legal_target_takes_damage() {
    let spell = card("Shock");
    let _rules = fixture_rules::activate(&[(spell, &DAMAGE_CHOICE)]);
    let mut game = scenario(P);
    let first = put(&mut game, ENEMY, "Gigantosaurus", Zone::Battlefield);
    let second = put(&mut game, ENEMY, "Gigantosaurus", Zone::Battlefield);
    let unrelated = put(&mut game, ENEMY, "Gigantosaurus", Zone::Battlefield);
    let cast_id = spawn(&mut game, P, spell, Zone::Hand);
    announce(
        &mut game,
        cast_id,
        &[Target::Card(first), Target::Card(second)],
    );
    let mut one_legal = game.clone();
    one_legal.move_object(first, Zone::Exile);
    one_legal.move_object(first, Zone::Battlefield);
    pass_pair(&mut one_legal);
    assert_eq!(one_legal.state.pending, Pending::Priority);
    assert_eq!(one_legal.object_view(first).damage, 0);
    assert_eq!(one_legal.object_view(second).damage, 5);
    assert!(
        !one_legal
            .object_view(second)
            .keywords
            .contains(Keywords::CANT_BLOCK)
    );

    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(ENEMY));
    let illegal = Action::ChooseEffectObject {
        object: game.object_ref(unrelated),
    };
    reject(&mut game, illegal);
    let checkpoint = game.snapshot();
    for (chosen, other) in [(first, second), (second, first)] {
        let action = Action::ChooseEffectObject {
            object: game.object_ref(chosen),
        };
        let mut branch = game.clone();
        let index = branch
            .legal_actions()
            .iter()
            .position(|a| a == &action)
            .unwrap();
        branch.apply_index(ENEMY, branch.revision(), index).unwrap();
        act(&mut game, action);
        assert_eq!(game.snapshot(), branch.snapshot());
        assert_eq!(game.object_view(chosen).damage, 5);
        assert_eq!(game.object_view(other).damage, 0);
        assert!(
            game.object_view(other)
                .keywords
                .contains(Keywords::CANT_BLOCK)
        );
        assert!(
            !game
                .object_view(chosen)
                .keywords
                .contains(Keywords::CANT_BLOCK)
        );
        game.restore(&checkpoint).unwrap();
    }
}

const MILLED_PICK: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(4)),
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::KindsOrLegendary(32),
                count: MilledCount::AnyNumber,
                destination: MilledDestination::HandAndCount,
            }),
            Instruction::IfAmountAtLeast {
                value: AmountValue::MovedCount,
                minimum: 1,
                skip: 1,
            },
            Instruction::Skip(1),
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};

#[test]
fn any_number_of_qualifying_cards_can_be_chosen_from_exact_public_mill_receipts() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &MILLED_PICK)]);
    for replacement in [false, true] {
        let mut game = scenario(P);
        let source = put(&mut game, P, "Grizzly Bears", Zone::Hand);
        let old = put(&mut game, P, "Banishing Light", Zone::Graveyard);
        let enchantment = put(&mut game, P, "Banishing Light", Zone::Library);
        let legendary = put(&mut game, P, "Ghalta, Primal Hunger", Zone::Library);
        let instant = put(&mut game, P, "Opt", Zone::Library);
        let land = put(&mut game, P, "Forest", Zone::Library);
        if replacement {
            put(&mut game, ENEMY, "Rest in Peace", Zone::Battlefield);
        }
        cast(&mut game, source, None);
        pass_pair(&mut game);
        pass_pair(&mut game);
        let selection = game.milled_selection_view(P).unwrap();
        assert_eq!(selection.milled.len(), 4);
        assert_eq!(selection.maximum, 2);
        assert_eq!(selection.minimum, 0);
        assert!(game.milled_selection_view(ENEMY).is_none());
        let destination = if replacement {
            Zone::Exile
        } else {
            Zone::Graveyard
        };
        for id in [enchantment, legendary, instant, land] {
            assert_eq!(game.object_view(id).zone, destination);
        }
        for id in [old, instant, land] {
            let action = Action::SelectMilled {
                object: game.object_ref(id),
            };
            reject(&mut game, action);
        }
        let mut none = game.clone();
        act(&mut none, Action::FinishMilledSelection);
        assert_eq!(none.counter_count(source, CounterKind::PlusOne), 1);
        let mut some = game.clone();
        act(
            &mut some,
            Action::SelectMilled {
                object: game.object_ref(legendary),
            },
        );
        act(&mut some, Action::FinishMilledSelection);
        assert_eq!(some.object_view(legendary).zone, Zone::Hand);
        assert_eq!(some.object_view(enchantment).zone, destination);
        for id in [legendary, enchantment] {
            let action = Action::SelectMilled {
                object: game.object_ref(id),
            };
            act(&mut game, action);
        }
        assert_eq!(game.object_view(legendary).zone, Zone::Hand);
        assert_eq!(game.object_view(enchantment).zone, Zone::Hand);
        assert_eq!(game.counter_count(source, CounterKind::PlusOne), 0);
    }
}

const GRAVE_FILTER: SearchFilter = SearchFilter::AnyOf(&[
    SearchFilter::Kinds(64),
    SearchFilter::AllOf(&[SearchFilter::Creature, SearchFilter::NonSubtype("Avatar")]),
]);
const GRAVE_RETURN: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(4)),
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ReturnGraveyardToHandMatching(GRAVE_FILTER),
                players: PlayerSet::Controller,
                filter: CREATURE_FILTER,
                amount: 1,
                optional_skip: Some(0),
            },
        ],
    ),
    ..CardRules::EMPTY
};
const CHOSEN_TYPE: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[StaticAbilityFragment::ChosenCreatureSubtypeAcrossZones],
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureType),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

#[test]
fn the_graveyard_choice_accepts_planeswalkers_or_current_non_avatar_creatures() {
    let spell = card("Opt");
    let type_provider = card("Banishing Light");
    let avatar = card("Overlord of the Hauntwoods");
    let _rules = fixture_rules::activate(&[
        (spell, &GRAVE_RETURN),
        (type_provider, &CHOSEN_TYPE),
        (avatar, &CardRules::EMPTY),
    ]);
    let mut game = scenario(P);
    let older = put(&mut game, P, "Grizzly Bears", Zone::Graveyard);
    let walker = put(&mut game, P, "Liliana, Dreadhorde General", Zone::Graveyard);
    let enemy = put(&mut game, ENEMY, "Grizzly Bears", Zone::Graveyard);
    let newcomer = put(&mut game, P, "Hill Giant", Zone::Library);
    let avatar = spawn(&mut game, P, avatar, Zone::Library);
    let cast_id = spawn(&mut game, P, spell, Zone::Hand);
    let mut changed_types = game.clone();
    let provider = spawn(&mut changed_types, P, type_provider, Zone::Battlefield);
    changed_types.set_entry_value(
        changed_types.object_ref(provider),
        EntryChoiceValue::CreatureType(CreatureType::from_name("Avatar").unwrap()),
    );
    for (game, allow_creatures) in [(&mut game, true), (&mut changed_types, false)] {
        cast(game, cast_id, None);
        pass_pair(game);
        assert_eq!(game.object_view(avatar).zone, Zone::Graveyard);
        assert_eq!(game.object_view(newcomer).zone, Zone::Graveyard);
        for (id, allowed) in [
            (older, allow_creatures),
            (newcomer, allow_creatures),
            (walker, true),
            (avatar, false),
            (enemy, false),
        ] {
            let action = Action::SelectGraveyardReturn {
                object: game.object_ref(id),
            };
            assert_eq!(game.legal_actions().contains(&action), allowed, "card {id}");
        }
        let chosen = Action::SelectGraveyardReturn {
            object: game.object_ref(walker),
        };
        act(game, chosen);
        assert_eq!(game.object_view(walker).zone, Zone::Hand);
    }
}
