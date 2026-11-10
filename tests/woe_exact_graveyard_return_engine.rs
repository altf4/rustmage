//! Sacrifice results bound a mandatory, non-targeted graveyard return batch.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const TOKEN: Card = Card::test_index(5286);
const CONQUEST: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::MaySacrificeAnyNumber(SacrificeFilter {
                kinds: 0,
                other: false,
                keywords: Keywords::NONE,
                source_only: false,
                nonland: false,
                tokens: None,
                artifact_enchantment_or_token: true,
                subtype: None,
            }),
            Instruction::ChooseGraveyardReturnCount(AmountValue::MovedCount),
        ],
    ),
    ..CardRules::EMPTY
};
const OBSERVER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDies {
            relation: CastRelation::You,
            other: true,
            nontoken: false,
            subtype: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    ..CardRules::EMPTY
};

fn begin(game: &mut Game, player: Player) -> CardId {
    let spell = put(game, player, "Divination", Zone::Hand);
    cast(game, spell, None);
    act(game, Action::Pass);
    act(game, Action::Pass);
    assert!(matches!(game.state.pending, Pending::EffectChoice { player: p } if p == player));
    spell
}

fn sacrifice(game: &mut Game, card: CardId) {
    let object = game.object_ref(card);
    act(game, Action::Sacrifice { object });
}

fn select_return(game: &Game, card: CardId) -> Action {
    Action::SelectGraveyardReturn {
        object: game.object_ref(card),
    }
}

#[test]
fn exact_public_choice_includes_fresh_artifact_creature_and_stages_all_entries() {
    let _rules = fixture_rules::activate(&[
        (Card::Divination, &CONQUEST),
        (Card::from_name("Bear Cub").unwrap(), &OBSERVER),
    ]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let observer = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let automaton = put(&mut game, player, "Adaptive Automaton", Zone::Battlefield);
        let offering = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let dragon = put(&mut game, player, "Shivan Dragon", Zone::Graveyard);
        let other = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
        let foreign = put(&mut game, opponent, "Shivan Dragon", Zone::Graveyard);
        let spell = begin(&mut game, player);
        sacrifice(&mut game, automaton);
        sacrifice(&mut game, offering);
        assert_eq!(game.object_view(automaton).zone, Zone::Battlefield);
        act(&mut game, Action::FinishEffectChoice);

        let choice = game.observe(opponent).effect_choice.unwrap();
        assert!(!choice.optional);
        assert_eq!(choice.amount, 2);
        assert_eq!(choice.required[player.index()], 2);
        assert_eq!(choice.required[opponent.index()], 0);
        assert_eq!(
            choice.candidates,
            [automaton, dragon, other].map(|id| game.object_ref(id))
        );
        assert_eq!(game.object_view(automaton).zone, Zone::Graveyard);
        assert_eq!(game.observe(player).group_move_count, Some(2));
        assert!(game.state.stack.is_empty());
        assert_eq!(game.state.players[player.index()].life, 20);
        reject(&mut game, Action::FinishGraveyardReturn);
        let wrong_owner = select_return(&game, foreign);
        reject(&mut game, wrong_owner);
        let noncreature = select_return(&game, offering);
        reject(&mut game, noncreature);

        let first = select_return(&game, dragon);
        act(&mut game, first.clone());
        reject(&mut game, first);
        reject(&mut game, Action::FinishGraveyardReturn);
        assert_eq!(game.object_view(dragon).zone, Zone::Graveyard);
        let action = select_return(&game, automaton);
        let checkpoint = game.snapshot();
        let revision = game.revision();
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        let mut indexed = game.clone();
        assert_eq!(
            indexed.apply(opponent, action.clone()),
            Err(Error::WrongActor)
        );
        assert_eq!(indexed.snapshot(), checkpoint);
        act(&mut game, action);
        indexed.apply_index(player, revision, index).unwrap();
        assert_eq!(indexed.snapshot(), game.snapshot());
        assert!(game.observe(player).effect_choice.is_none());
        assert_eq!(game.object_view(automaton).zone, Zone::Graveyard);
        assert_eq!(game.object_view(dragon).zone, Zone::Graveyard);
        let chosen = Action::ChooseEntryValue {
            value: EntryChoiceValue::CreatureType(CreatureType::from_name("Dragon").unwrap()),
        };
        for branch in [&mut game, &mut indexed] {
            act(branch, chosen.clone());
            assert_eq!(branch.object_view(automaton).zone, Zone::Battlefield);
            assert_eq!(branch.object_view(dragon).zone, Zone::Battlefield);
            assert_eq!(branch.object_view(dragon).power, 6);
            assert_eq!(branch.object_view(other).zone, Zone::Graveyard);
            assert_eq!(branch.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(
                branch.state.stack.last().unwrap().source.unwrap().id,
                observer
            );
            assert_eq!(branch.state.players[player.index()].life, 20);
            settle(branch);
            assert_eq!(branch.state.players[player.index()].life, 21);
        }
        assert_eq!(indexed.snapshot(), game.snapshot());

        indexed.restore(&checkpoint).unwrap();
        let alternate = select_return(&indexed, other);
        act(&mut indexed, alternate);
        settle(&mut indexed);
        assert_eq!(indexed.object_view(automaton).zone, Zone::Graveyard);
        assert_eq!(indexed.object_view(other).zone, Zone::Battlefield);
    }
}

#[test]
fn zero_and_replaced_sacrifices_and_fewer_eligible_cards_set_the_exact_quota() {
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|slot| slot.set(None));
        }
    }
    let _reset = Reset;
    let _rules = fixture_rules::activate(&[(Card::Divination, &CONQUEST)]);
    for player in [Player::P0, Player::P1] {
        for available in [0, 1, 3] {
            let mut game = scenario(player);
            let exiled = put(&mut game, player, "Short Bow", Zone::Battlefield);
            let prevented = put(&mut game, player, "Adaptive Automaton", Zone::Battlefield);
            let token = game.state.objects.len() as CardId;
            game.create_tokens(player, TOKEN, 1, false);
            let graveyard: Vec<_> = (0..available)
                .map(|_| put(&mut game, player, "Llanowar Elves", Zone::Graveyard))
                .collect();
            game.add_counter(exiled, CounterKind::Finality, 1);
            zone_replacements::TEST_PREVENT_ZONE_MOVE
                .with(|slot| slot.set(Some(game.object_ref(prevented))));
            let spell = begin(&mut game, player);
            let mut zero = game.clone();
            act(&mut zero, Action::FinishEffectChoice);
            assert_eq!(zero.object_view(spell).zone, Zone::Graveyard);
            assert!(zero.observe(player).effect_choice.is_none());
            assert_eq!(zero.object_view(exiled).zone, Zone::Battlefield);
            for &card in &graveyard {
                assert_eq!(zero.object_view(card).zone, Zone::Graveyard);
            }

            for card in [exiled, prevented, token] {
                sacrifice(&mut game, card);
            }
            act(&mut game, Action::FinishEffectChoice);
            assert_eq!(game.object_view(exiled).zone, Zone::Exile);
            assert_eq!(game.object_view(prevented).zone, Zone::Battlefield);
            assert_eq!(
                game.state.players[player.index()].permanents_sacrificed_this_turn,
                2
            );
            let expected = available.min(2);
            if expected == 0 {
                assert!(game.observe(player).effect_choice.is_none());
            } else {
                let choice = game.observe(player).effect_choice.unwrap();
                assert_eq!(choice.amount, expected as u32);
                assert!(!choice.optional);
                for &card in graveyard.iter().take(expected) {
                    let action = select_return(&game, card);
                    act(&mut game, action);
                }
            }
            assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
            for (index, card) in graveyard.into_iter().enumerate() {
                assert_eq!(
                    game.object_view(card).zone,
                    if index < expected {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    }
                );
            }
        }
    }
}
