//! Multi-target copiable exceptions, temporary haste, and exact end-step exile.
use crate::support::*;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};

const INFILTRATE: CardRules = CardRules {
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
        &[Instruction::CreateTemporaryCopiesOfEachTarget {
            exceptions: CopyExceptions {
                add_keywords: Keywords::CHANGELING,
                ..CopyExceptions::NONE
            },
            keywords: Keywords::HASTE,
            exile_ability: 0,
        }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileDelayedBattlefieldObjects],
    }],
    ..CardRules::EMPTY
};

fn tokens(game: &Game) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.controller == Player::P0
                && object.is_token())
            .then_some(id as CardId)
        })
        .collect()
}

#[test]
fn every_surviving_target_gets_its_own_hasty_changeling_copy_then_exile() {
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &INFILTRATE)]);
    for (depart, replaced_types) in [(false, false), (true, false), (false, true)] {
        let mut game = scenario(Player::P0);
        let first = put(&mut game, Player::P1, "Bear Cub", Zone::Battlefield);
        let second = put(&mut game, Player::P1, "Hill Giant", Zone::Battlefield);
        if replaced_types {
            let mut recipe = CopyRecipe::new(Card::GrizzlyBears);
            recipe.overrides = Some(CopyOverrides {
                types: Some(CopyTypes {
                    replace: true,
                    ..CopyTypes::default()
                }),
                ..CopyOverrides::NONE
            });
            game.apply_temporary_copy(game.object_ref(first), recipe, CopyDeadline::EndOfTurn);
        }
        let own = put(&mut game, Player::P0, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut game, Player::P0, "Opt", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        let before = game.snapshot();
        let revision = game.revision();
        let first_choice = Action::ChooseTarget {
            target: Target::Card(first),
        };
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == first_choice)
            .unwrap();
        let mut indexed = game.clone();
        indexed.apply_index(Player::P0, revision, index).unwrap();
        act(&mut game, first_choice);
        assert_eq!(game.snapshot(), indexed.snapshot());
        indexed.restore(&before).unwrap();
        assert_eq!(indexed.snapshot(), before);
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(second),
            },
        );
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        pay(&mut game);
        if depart {
            game.move_object(second, Zone::Graveyard);
        }
        settle(&mut game);
        let created = tokens(&game);
        assert_eq!(created.len(), if depart { 1 } else { 2 });
        for &id in &created {
            assert!(
                game.keywords(id)
                    .contains(Keywords::CHANGELING.union(Keywords::HASTE))
            );
            assert!(game.has_subtype(id, "Vulcan"));
            assert!(game.has_subtype(id, "Borg"));
            assert_eq!(game.counter_count(id, CounterKind::PlusOne), 0);
        }
        let first_copy = if replaced_types {
            Card::GrizzlyBears
        } else {
            Card::from_name("Bear Cub").unwrap()
        };
        assert!(
            created
                .iter()
                .any(|&id| game.object_view(id).card == first_copy)
        );
        if !depart {
            assert!(
                created
                    .iter()
                    .any(|&id| game.object_view(id).card == Card::from_name("Hill Giant").unwrap())
            );
        }
        assert_eq!(game.observe(Player::P0).delayed_steps.unwrap().len(), 1);
        // The delayed trigger can fail to exile a token. Haste has no printed
        // duration, so cleanup alone must not remove it from a survivor.
        let mut survivor = game.clone();
        survivor.state.step = Step::Cleanup;
        survivor.finish_cleanup();
        for &id in &created {
            assert!(survivor.keywords(id).contains(Keywords::HASTE));
        }
        game.begin_step(Step::End);
        game.prepare_triggers();
        settle(&mut game);
        assert!(
            created
                .into_iter()
                .all(|id| game.object_view(id).zone != Zone::Battlefield)
        );
    }
}

#[test]
fn noncreature_copy_with_changeling_has_no_creature_subtypes_in_entry_preview() {
    let mut game = scenario(Player::P0);
    let artifact = put(&mut game, Player::P0, "Short Bow", Zone::Hand);
    let mut view = game.object_view(artifact);
    view.copy_recipe = Some(CopyRecipe::new(view.card).with_exceptions(CopyExceptions {
        add_keywords: Keywords::CHANGELING,
        ..CopyExceptions::NONE
    }));
    let entry =
        rustmage_core::engine::entries::EntryMove::from_view(view, Zone::Battlefield, false);
    assert!(!Game::entry_replacement_subtypes(&entry).contains("Vulcan"));
    assert!(!Game::entry_choice_subtypes(&entry).contains("Borg"));
}
