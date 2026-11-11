//! The target's battlefield name and controller bind a private resolution search.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const EXILE_SEARCH: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::ExileTargetAndSearchItsController],
    ),
    ..CardRules::EMPTY
};

const BODYGUARD: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Filtered {
                base: &TargetRule::Filtered {
                    base: &TargetRule::Creature,
                    predicate: &TargetPredicate::OtherThanSource,
                },
                predicate: &TargetPredicate::WithoutSubtype("Fox"),
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::ExileUntilSourceLeaves],
    }],
    ..CardRules::EMPTY
};

#[test]
fn exile_search_uses_the_pre_exile_controller_and_draws_for_actual_hand_exiles() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Murder").unwrap(), &EXILE_SEARCH)]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let other = player.other();
        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        game.state.objects[creature as usize].controller = other;
        let own_match = put(&mut game, player, "Bear Cub", Zone::Hand);
        let hand_match = put(&mut game, other, "Bear Cub", Zone::Hand);
        let grave_match = put(&mut game, other, "Bear Cub", Zone::Graveyard);
        let library_match = put(&mut game, other, "Bear Cub", Zone::Library);
        let other_name = put(&mut game, other, "Hill Giant", Zone::Hand);
        let spell = put(&mut game, player, "Murder", Zone::Hand);
        cast(&mut game, spell, Some(Target::Card(creature)));
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);

        assert_eq!(game.object_view(creature).zone, Zone::Exile);
        assert_eq!(game.actor(), Some(player));
        assert!(game.observe(player).effect_choice.is_some());
        assert!(game.observe(other).effect_choice.is_none());
        for id in [own_match, other_name] {
            assert!(
                !game
                    .legal_actions()
                    .contains(&Action::SelectResolutionExile {
                        object: game.object_ref(id),
                    })
            );
        }
        for id in [hand_match, grave_match, library_match] {
            let object = game.object_ref(id);
            act(&mut game, Action::SelectResolutionExile { object });
        }

        let checkpoint = game.snapshot();
        let revision = game.revision();
        let finish = game
            .legal_actions()
            .iter()
            .position(|a| *a == Action::FinishSearch)
            .unwrap();
        let mut index = game.clone();
        index.apply_index(player, revision, finish).unwrap();
        act(&mut game, Action::FinishSearch);
        assert_eq!(game.snapshot(), index.snapshot());
        assert_eq!(game.state.players[other.index()].cards_drawn_this_turn, 1);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
        for id in [hand_match, grave_match, library_match] {
            assert_eq!(game.object_view(id).zone, Zone::Exile);
        }
        assert_eq!(game.object_view(own_match).zone, Zone::Hand);
        assert_eq!(game.object_view(other_name).zone, Zone::Hand);
        index.restore(&checkpoint).unwrap();
        act(&mut index, Action::FinishSearch);
        assert_eq!(game.snapshot(), index.snapshot());
    }
}

#[test]
fn the_linked_entry_choice_finishes_before_the_private_search() {
    let bodyguard = Card::from_name("Werefox Bodyguard").unwrap();
    let the_end = Card::from_name("The End").unwrap();
    let _rules = fixture_rules::activate(&[(bodyguard, &BODYGUARD), (the_end, &EXILE_SEARCH)]);
    for player in [Player::P0, Player::P1] {
        let other = player.other();
        let mut game = scenario(other);
        let source = spawn(&mut game, other, bodyguard, Zone::Hand);
        let hand_match = spawn(&mut game, other, bodyguard, Zone::Hand);
        let own_match = spawn(&mut game, player, bodyguard, Zone::Hand);
        let spell = spawn(&mut game, player, the_end, Zone::Hand);
        let automaton = put(&mut game, other, "Adaptive Automaton", Zone::Battlefield);

        act(&mut game, Action::AnnounceCast { card: source });
        act(
            &mut game,
            Action::PayCosts {
                payment: [2, 0, 0, 0, 0, 1],
            },
        );
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(automaton),
            },
        );
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        settle(&mut game);
        assert_eq!(game.object_view(source).zone, Zone::Battlefield);
        assert_eq!(game.object_view(automaton).zone, Zone::Exile);
        let exiled_automaton = game.object_ref(automaton);

        act(&mut game, Action::Pass);
        act(&mut game, Action::AnnounceCast { card: spell });
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 2, 0, 0, 2],
            },
        );
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        assert_eq!(game.object_view(source).zone, Zone::Exile);
        assert_eq!(game.object_view(automaton).zone, Zone::Exile);
        assert_eq!(game.actor(), Some(other));
        assert!(game.observe(player).effect_choice.is_none());
        assert!(game.observe(other).effect_choice.is_none());

        let entry = Action::ChooseEntryValue {
            value: EntryChoiceValue::CreatureType(CreatureType::from_name("Human").unwrap()),
        };
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == entry)
            .unwrap();
        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        act(&mut game, entry.clone());
        indexed.apply_index(other, revision, index).unwrap();
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&checkpoint).unwrap();
        act(&mut indexed, entry);
        assert_eq!(game.snapshot(), indexed.snapshot());

        let returned = game.object_view(automaton);
        assert_eq!(returned.zone, Zone::Battlefield);
        assert_eq!(returned.controller, other);
        assert_ne!(returned.reference, exiled_automaton);
        assert_eq!(game.actor(), Some(player));
        assert!(game.observe(player).effect_choice.is_some());
        assert!(game.observe(other).effect_choice.is_none());
        assert_eq!(game.state.players[other.index()].cards_drawn_this_turn, 0);

        act(&mut indexed, Action::FinishSearch);
        assert_eq!(indexed.object_view(hand_match).zone, Zone::Hand);
        assert_eq!(
            indexed.state.players[other.index()].cards_drawn_this_turn,
            0
        );
        assert_eq!(indexed.object_view(spell).zone, Zone::Graveyard);

        let select = Action::SelectResolutionExile {
            object: game.object_ref(hand_match),
        };
        let private = game.snapshot();
        assert_eq!(game.apply(other, select.clone()), Err(Error::WrongActor));
        assert_eq!(game.snapshot(), private);
        let wrong_owner = Action::SelectResolutionExile {
            object: game.object_ref(own_match),
        };
        reject(&mut game, wrong_owner);
        act(&mut game, select);
        act(&mut game, Action::FinishSearch);
        assert_eq!(game.object_view(hand_match).zone, Zone::Exile);
        assert_eq!(game.object_view(own_match).zone, Zone::Hand);
        assert_eq!(game.state.players[other.index()].cards_drawn_this_turn, 1);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
        assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    }
}
