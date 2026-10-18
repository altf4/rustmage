//! Admitted runtime recipes exercised through production effects and owned branches.
use super::ability_timeline_tests::{cast, checked, settle, setup};
use super::tests::spawn;
use super::*;
use crate::{ChoiceColor, CreatureType};

fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn copies(g: &Game, name: &str) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.is_copy_token() && o.card.name() == name)
        .map(|(i, _)| i as CardId)
        .collect()
}
fn reflect(g: &mut Game, p: Player, target: CardId) {
    let id = fixture(g, p, "Self-Reflection", Zone::Hand);
    cast(g, id, None, &[Target::Card(target)]);
    settle(g);
}
#[test]
fn runtime_copy_paid_values_are_copiable_and_token_identity_survives_owned_restore() {
    assert_eq!(std::mem::size_of::<Object>(), 144);
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        assert!(g.state.copy_recipes.is_none());
        let bear = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[bear as usize].growth = Stats {
            power: 7,
            toughness: 7,
        };
        g.state.objects[bear as usize].counters.plus_one = 3;
        let armor = fixture(&mut g, p, "Celestial Armor", Zone::Battlefield);
        let equip = g.legal_actions().into_iter().find(|a| matches!(a, Action::Activate { card, target: Some(Target::Card(t)), .. } if *card == armor && *t == bear)).unwrap();
        checked(&mut g, equip);
        settle(&mut g);
        assert!(g.keywords(bear).contains(Keywords::FLYING));
        reflect(&mut g, p, bear);
        let copy = copies(&g, "Bear Cub")[0];
        let view = g.object_view(copy);
        assert_eq!(view.name(), "Bear Cub");
        assert!(view.is_token());
        assert_eq!(
            view.copy_recipe,
            Some(CopyRecipe::new(Card::from_name("Bear Cub").unwrap()))
        );
        assert_eq!((g.stats(copy).power, g.stats(copy).toughness), (2, 2));
        assert!(!g.keywords(copy).contains(Keywords::FLYING));
        assert_eq!(g.object_view(bear).copy_recipe, None);
        reflect(&mut g, p, copy);
        assert_eq!(copies(&g, "Bear Cub").len(), 2);
        let before = g.snapshot();
        let mut child = {
            let parent = g.clone();
            parent.clone()
        };
        child.sacrifice_batch(&[(p, copy)]);
        child.state_based_actions();
        child.give_priority();
        settle(&mut child);
        assert_eq!(child.object_view(copy).zone, Zone::Outside);
        assert!(child.copy_recipe(child.object_ref(copy)).is_none());
        let ceased = child.snapshot();
        for destination in [
            Zone::Battlefield,
            Zone::Hand,
            Zone::Graveyard,
            Zone::Library,
        ] {
            assert!(!child.move_object(copy, destination).moved);
            assert_eq!(child.snapshot(), ceased, "ceased token cannot move again");
        }
        assert_eq!(g.snapshot(), before);
        child.restore(&before).unwrap();
        assert_eq!(child.snapshot(), before);
        assert!(child.object_view(copy).is_token());
        checked(&mut child, Action::Pass);
    }
}
#[test]
fn runtime_copy_enters_with_fresh_printed_counters_not_source_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = fixture(&mut g, p, "Myojin of Night's Reach", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::Divinity), 1);
        g.stage_copy_creation(p, CopyRecipe::new(g.state.objects[source as usize].card), 1);
        let copy = copies(&g, "Myojin of Night's Reach")[0];
        assert_eq!(g.counter_count(copy, CounterKind::Divinity), 0);
        g.state_based_actions();
        let object = g.object_ref(copy);
        checked(&mut g, Action::KeepLegend { object });
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.object_view(copy).zone, Zone::Battlefield);
        assert!(!g.keywords(copy).contains(Keywords::INDESTRUCTIBLE));
    }
}

fn resolve_to(g: &mut Game, wanted: impl Fn(&Pending) -> bool) {
    g.prepare_triggers();
    for _ in 0..80 {
        if wanted(&g.state.pending) {
            return;
        }
        let actions = g.legal_actions();
        let action = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::PutTrigger { .. } | Action::FinishTargets | Action::PayCosts { .. }
                    )
                })
                .unwrap()
        };
        checked(g, action);
    }
    panic!("missing decision {:?}", g.state.pending);
}
#[test]
fn runtime_copy_kicker_simultaneous_replacements_and_fixed_offspring_recipe() {
    for p in [Player::P0, Player::P1] {
        for kicked in [false, true] {
            let mut g = setup(p);
            fixture(&mut g, p, "Doubling Season", Zone::Battlefield);
            let offspring = Card::catalogue()
                .find(|c| c.is_token() && c.name() == "Tender Wildguide")
                .unwrap();
            let target = spawn(&mut g, p.other(), offspring, Zone::Battlefield);
            let rite = fixture(&mut g, p, "Rite of Replication", Zone::Hand);
            let before: u32 = g.state.players[p.index()].mana.iter().sum();
            checked(&mut g, Action::AnnounceCast { card: rite });
            checked(&mut g, Action::ChooseKicker { pay: kicked });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                checked(&mut g, Action::FinishTargets);
            }
            let pay = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, pay);
            settle(&mut g);
            assert_eq!(
                before - g.state.players[p.index()].mana.iter().sum::<u32>(),
                if kicked { 9 } else { 4 }
            );
            let created = copies(&g, "Tender Wildguide");
            assert_eq!(created.len(), if kicked { 10 } else { 2 });
            for id in created {
                assert_eq!((g.stats(id).power, g.stats(id).toughness), (1, 1));
                assert_eq!(g.object_view(id).copy_recipe.unwrap().definition, offspring);
                assert!(!g.state.objects[id as usize].card.activations().is_empty());
            }
        }
    }
}
#[test]
fn runtime_copy_animated_land_copies_underlying_land_and_mana_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = fixture(&mut g, p, "Forest", Zone::Battlefield);
        // Existing layer-four animation overlay models a temporary animation;
        // its noncopiable types and pump must not become the token's recipe.
        g.state.crew = Some(Box::new(crew::CrewState {
            animated: vec![g.object_ref(land)],
            ..Default::default()
        }));
        g.state.objects[land as usize].growth = Stats {
            power: 3,
            toughness: 3,
        };
        assert!(g.is_creature(land));
        reflect(&mut g, p, land);
        let copy = copies(&g, "Forest")[0];
        assert!(!g.is_creature(copy));
        assert!(g.state.objects[copy as usize].card.is_land());
        let before = g.state.players[p.index()].mana[ChoiceColor::Green.index()];
        checked(&mut g, Action::TapMana { card: copy });
        assert_eq!(
            g.state.players[p.index()].mana[ChoiceColor::Green.index()],
            before + 1
        );
    }
}
#[test]
fn runtime_copy_fresh_entry_choices_and_counter_replacements() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let automaton = fixture(&mut g, p, "Adaptive Automaton", Zone::Hand);
        cast(&mut g, automaton, None, &[]);
        resolve_to(&mut g, |pending| {
            matches!(pending, Pending::EntryValue { .. })
        });
        let bear = EntryChoiceValue::CreatureType(CreatureType::from_name("Bear").unwrap());
        checked(&mut g, Action::ChooseEntryValue { value: bear });
        settle(&mut g);
        let reflection = fixture(&mut g, p, "Self-Reflection", Zone::Hand);
        cast(&mut g, reflection, None, &[Target::Card(automaton)]);
        resolve_to(&mut g, |pending| {
            matches!(pending, Pending::EntryValue { .. })
        });
        let human = EntryChoiceValue::CreatureType(CreatureType::from_name("Human").unwrap());
        checked(&mut g, Action::ChooseEntryValue { value: human });
        settle(&mut g);
        let copy = copies(&g, "Adaptive Automaton")[0];
        assert_eq!(g.object_view(copy).entry_choice, Some(human));
        assert!(!g.has_subtype(copy, "Bear"));
        assert!(g.has_subtype(copy, "Human"));
        fixture(&mut g, p, "Doubling Season", Zone::Battlefield);
        // The copied walker gets fresh printed loyalty, not source loyalty.
        g.stage_copy_creation(
            p,
            CopyRecipe::new(Card::from_name("Liliana, Dreadhorde General").unwrap()),
            1,
        );
        g.advance_replacements(false);
        g.advance_entries();
        g.state_based_actions();
        settle(&mut g);
        let walkers = copies(&g, "Liliana, Dreadhorde General");
        assert_eq!(walkers.len(), 1);
        for id in walkers {
            assert_eq!(g.counter_count(id, CounterKind::Loyalty), 12);
        }
    }
}
#[test]
fn runtime_copy_aura_chooses_nontargeted_attachment_and_impossible_entry_creates_nothing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p.other());
        let bear = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let armor = fixture(&mut g, p.other(), "Celestial Armor", Zone::Hand);
        cast(&mut g, armor, None, &[]);
        settle(&mut g);
        assert!(g.keywords(bear).contains(Keywords::HEXPROOF));
        let pacifism = Card::from_name("Pacifism").unwrap();
        g.stage_copy_creation(p, CopyRecipe::new(pacifism), 1);
        assert!(g.advance_entries());
        let object = g.object_ref(bear);
        checked(&mut g, Action::ChooseAuraAttachment { object });
        settle(&mut g);
        let copy = copies(&g, "Pacifism")[0];
        assert_eq!(g.object_view(copy).attached_to, Some(object));
        let mut empty = setup(p);
        empty.stage_copy_creation(p, CopyRecipe::new(pacifism), 1);
        empty.advance_entries();
        assert!(copies(&empty, "Pacifism").is_empty());
        assert!(empty.state.copy_recipes.is_none());
        assert_eq!(empty.state.free_tokens.len(), 1);
        // Reuse the impossible token slot without retaining its Aura factory.
        empty.stage_copy_creation(p, CopyRecipe::new(Card::Forest), 1);
        assert_eq!(copies(&empty, "Forest").len(), 1);
        assert!(empty.state.free_tokens.is_empty());
    }
}
#[test]
fn runtime_copy_horde_exact_second_draw_and_departed_source_lki() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.draw(p);
        let source = fixture(&mut g, p, "Homunculus Horde", Zone::Hand);
        cast(&mut g, source, None, &[]);
        settle(&mut g);
        g.draw(p);
        g.prepare_triggers();
        assert_eq!(g.state.stack.len(), 1);
        let event = g.move_object(source, Zone::Graveyard);
        assert!(event.moved);
        settle(&mut g);
        let first = copies(&g, "Homunculus Horde")[0];
        assert_eq!(g.stats(first).power, 2);
        g.draw(p);
        settle(&mut g);
        assert_eq!(copies(&g, "Homunculus Horde").len(), 1);
        g.state.players[p.index()].cards_drawn_this_turn = 0;
        g.draw(p);
        g.draw(p);
        settle(&mut g);
        assert_eq!(copies(&g, "Homunculus Horde").len(), 2);
    }
}
#[test]
fn runtime_copy_flashback_and_replication_paid_upkeep_target_rules() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let reflection = fixture(&mut g, p, "Self-Reflection", Zone::Graveyard);
        checked(&mut g, Action::AnnounceFlashback { card: reflection });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert_eq!(g.object_view(reflection).zone, Zone::Exile);
        let rep = fixture(&mut g, p, "Extravagant Replication", Zone::Hand);
        cast(&mut g, rep, None, &[]);
        settle(&mut g);
        let rock = fixture(&mut g, p, "Hedron Archive", Zone::Battlefield);
        g.begin_step(Step::Upkeep);
        g.prepare_triggers();
        assert!(g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(rock)
        }));
        assert!(!g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(rep)
        }));
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(rock),
            },
        );
        settle(&mut g);
        let copy = copies(&g, "Hedron Archive")[0];
        assert!(g.state.objects[copy as usize].card.is_artifact());
        assert_eq!(g.stats(copy).power, 0);
    }
}
#[test]
fn runtime_copy_illegal_enemy_target_is_atomic_and_departed_target_fizzles() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let reflection = fixture(&mut g, p, "Self-Reflection", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: reflection });
        let before = g.snapshot();
        let revision = g.revision();
        assert_eq!(
            g.apply(
                p,
                Action::ChooseTarget {
                    target: Target::Card(enemy)
                }
            ),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), revision);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        let bounce = fixture(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(own)]);
        settle(&mut g);
        assert_eq!(g.object_view(own).zone, Zone::Hand);
        assert!(copies(&g, "Bear Cub").is_empty());
        assert!(g.state.copy_recipes.is_none());
        assert_eq!(g.object_view(reflection).zone, Zone::Graveyard);
    }
}
#[test]
fn runtime_copy_token_filters_frozen_damage_facts_and_speculative_bargain_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let definition = Card::from_name("Bear Cub").unwrap();
        g.stage_copy_creation(p, CopyRecipe::new(definition), 1);
        let token = copies(&g, "Bear Cub")[0];
        let physical = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let source = fixture(&mut g, p, "High Fae Negotiator", Zone::Hand);
        let nontoken = TargetRule::Filtered {
            base: &TargetRule::Creature,
            predicate: &TargetPredicate::Nontoken,
        };
        assert!(!g.target_matches(nontoken, p, g.object_ref(source), Target::Card(token)));
        assert!(g.target_matches(nontoken, p, g.object_ref(source), Target::Card(physical)));
        let facts = g.damage_facts(token);
        assert!(facts.is_token());
        assert!(!g.damage_facts(physical).is_token());
        checked(&mut g, Action::AnnounceCast { card: source });
        let before = g.snapshot();
        let revision = g.revision();
        let rejected = Action::ChooseBargain {
            sacrifice: Some(g.object_ref(physical)),
        };
        assert_eq!(g.apply(p, rejected), Err(Error::IllegalAction));
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.revision(), revision);
        let sacrifice = Some(g.object_ref(token));
        checked(&mut g, Action::ChooseBargain { sacrifice });
        while g.state.announcing.is_some() {
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                .unwrap();
            checked(&mut g, action);
        }
        settle(&mut g);
        assert_eq!(g.object_view(token).zone, Zone::Outside);
        assert!(g.state.copy_recipes.is_none());
        assert!(facts.is_token());
        assert_eq!(g.state.players[p.index()].life, STARTING_LIFE + 3);
    }
}
