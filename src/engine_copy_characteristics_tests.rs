//! Copiable exceptions run through ordinary paid spells, layers and owned triggers.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::*;

fn exception_copy(g: &mut Game, p: Player, exceptions: CopyExceptions) -> CardId {
    let recipe = CopyRecipe {
        definition: Card::from_name("Bear Cub").unwrap(),
        exceptions,
    };
    g.stage_copy_creation(p, recipe, 1);
    settle(g);
    g.state
        .copy_recipes
        .as_ref()
        .unwrap()
        .iter()
        .rev()
        .find(|r| r.recipe.exceptions == exceptions && g.on_battlefield(r.object.card))
        .unwrap()
        .object
        .card
}
#[test]
fn copiable_exceptions_survive_copying_but_obey_later_type_and_ability_layers() {
    assert_eq!(std::mem::size_of::<Object>(), 144);
    assert_eq!(std::mem::size_of::<Pending>(), 40);
    assert_eq!(std::mem::size_of::<ManaProduction>(), 32);
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let exceptions = CopyExceptions::HASTE_AND_SACRIFICE.union(CopyExceptions::NIGHTMARE);
        let source = exception_copy(&mut g, p, exceptions);
        assert!(g.keywords(source).contains(Keywords::HASTE));
        assert!(g.has_subtype(source, "Nightmare"));
        assert!(g.has_subtype(source, "Bear"));
        let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, None, &[Target::Card(source)]);
        settle(&mut g);
        assert!(!g.printed_abilities_available(source));
        assert!(!g.keywords(source).contains(Keywords::HASTE));
        assert!(!g.has_subtype(source, "Nightmare"));
        let reflection = named(&mut g, p, "Self-Reflection", Zone::Hand);
        cast(&mut g, reflection, None, &[Target::Card(source)]);
        settle(&mut g);
        let copy = g
            .state
            .copy_recipes
            .as_ref()
            .unwrap()
            .iter()
            .find(|r| r.object.card != source)
            .unwrap()
            .object
            .card;
        assert_eq!(
            g.copy_recipe(g.object_ref(copy)).unwrap().exceptions,
            exceptions
        );
        assert!(g.keywords(copy).contains(Keywords::HASTE));
        assert!(g.has_subtype(copy, "Nightmare"));
        assert_eq!(
            g.stats(copy),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(
            g.observe(p.other())
                .objects
                .iter()
                .all(|o| o.zone != Zone::Hand)
        );
        g.begin_step(Step::End);
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .filter(|s| s.kind == StackKind::CopyEndStepSacrifice)
                .count(),
            1
        );
        settle(&mut g);
        assert!(g.on_battlefield(source));
        assert_eq!(g.state.objects[copy as usize].zone, Zone::Outside);
        g.move_object(aura, Zone::Graveyard);
        g.begin_step(Step::End);
        settle(&mut g);
        assert_eq!(g.state.objects[source as usize].zone, Zone::Outside);
    }
}
#[test]
fn token_current_controller_owns_its_trigger_and_later_control_change_prevents_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let token = exception_copy(&mut g, p, CopyExceptions::HASTE_AND_SACRIFICE);
        let offering = named(&mut g, p, "Harmless Offering", Zone::Hand);
        cast(
            &mut g,
            offering,
            None,
            &[Target::Player(p.other()), Target::Card(token)],
        );
        settle(&mut g);
        assert_eq!(g.state.objects[token as usize].controller, p.other());
        g.begin_step(Step::End);
        let packet = g
            .state
            .pending_triggers
            .iter()
            .find(|s| s.kind == StackKind::CopyEndStepSacrifice)
            .unwrap();
        assert_eq!(packet.controller, p.other());
        assert_eq!(packet.object, g.object_ref(token));
        let transfer = g.activation_item(p.other(), token, 0);
        g.gain_control(&transfer, Subject::Source, PlayerSet::Opponent);
        settle(&mut g);
        assert!(g.on_battlefield(token));
        g.begin_step(Step::End);
        settle(&mut g);
        assert_eq!(g.state.objects[token as usize].zone, Zone::Outside);
    }
}
#[test]
fn a_trigger_already_on_the_stack_survives_ability_loss_and_never_rebinds_a_new_slot() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let token = exception_copy(&mut g, p, CopyExceptions::HASTE_AND_SACRIFICE);
        g.begin_step(Step::End);
        g.prepare_triggers();
        assert!(
            g.state
                .stack
                .iter()
                .any(|s| s.kind == StackKind::CopyEndStepSacrifice)
        );
        let old = g.object_ref(token);
        g.move_object(token, Zone::Exile);
        g.state_based_actions();
        let replacement = exception_copy(&mut g, p, CopyExceptions::NIGHTMARE);
        assert_eq!(replacement, token);
        assert_ne!(g.object_ref(replacement), old);
        assert!(g.on_battlefield(replacement));
        // The previous end-step packet resolves using its exact source, not the reused slot.
        settle(&mut g);
        assert!(g.on_battlefield(replacement));
        let live = exception_copy(&mut g, p, CopyExceptions::HASTE_AND_SACRIFICE);
        g.begin_step(Step::End);
        g.prepare_triggers();
        let witness = named(&mut g, p, "Witness Protection", Zone::Hand);
        let mut entry = g.entry_move(witness, Zone::Battlefield, false);
        entry.attachment = Some(g.object_ref(live));
        g.stage_entries(vec![entry], None);
        assert!(!g.printed_abilities_available(live));
        settle(&mut g);
        assert_eq!(g.state.objects[live as usize].zone, Zone::Outside);
    }
}
#[test]
fn source_copy_recipe_is_owned_lki_and_created_after_the_end_step_waits_for_the_next_one() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.begin_step(Step::End);
        let token = exception_copy(&mut g, p, CopyExceptions::HASTE_AND_SACRIFICE);
        assert!(
            !g.state
                .pending_triggers
                .iter()
                .any(|s| s.kind == StackKind::CopyEndStepSacrifice)
        );
        let item = g.activation_item(p, token, 0);
        let before = g.snapshot();
        g.move_object(token, Zone::Exile);
        g.state_based_actions();
        let recipe = g.recipe_for_subject(&item, Subject::Source).unwrap();
        assert_eq!(recipe.exceptions, CopyExceptions::HASTE_AND_SACRIFICE);
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        branch.stage_copy_creation(p, recipe, 1);
        settle(&mut branch);
        let copy = branch.state.copy_recipes.as_ref().unwrap()[0].object.card;
        assert!(branch.keywords(copy).contains(Keywords::HASTE));
        branch.begin_step(Step::End);
        settle(&mut branch);
        assert_eq!(branch.state.objects[copy as usize].zone, Zone::Outside);
        g.restore(&before).unwrap();
        assert!(g.on_battlefield(token));
        checked(&mut g, Action::Pass);
    }
}

#[test]
fn gated_electroduplicate_pays_normal_and_flashback_costs_and_creates_copiable_abilities() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let other = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[source as usize].growth = Stats {
            power: 5,
            toughness: 5,
        };
        let spell = super::tests::spawn(
            &mut g,
            p,
            Card::from_name("Electroduplicate").unwrap(),
            Zone::Hand,
        );
        g.state.players[p.index()].mana = [0, 0, 0, 3, 0, 0];
        checked(&mut g, Action::AnnounceCast { card: spell });
        let before = g.snapshot();
        assert_eq!(
            g.apply(
                p,
                Action::ChooseTarget {
                    target: Target::Card(other)
                }
            ),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), before);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        while g.state.announcing.is_some() {
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                .unwrap();
            checked(&mut g, action);
        }
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
        let first = g.state.copy_recipes.as_ref().unwrap()[0].object.card;
        assert_eq!(
            g.stats(first),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(g.keywords(first).contains(Keywords::HASTE));
        g.begin_step(Step::End);
        settle(&mut g);
        assert_eq!(g.state.objects[first as usize].zone, Zone::Outside);
        g.state.step = Step::Main1;
        g.state.priority = p;
        g.state.players[p.index()].mana = [0, 0, 0, 4, 0, 0];
        checked(&mut g, Action::AnnounceFlashback { card: spell });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        while g.state.announcing.is_some() {
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
                .unwrap();
            checked(&mut g, action);
        }
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Exile);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceFlashback { card: spell })
        );
        let second = g.state.copy_recipes.as_ref().unwrap()[0].object.card;
        assert_eq!(
            g.stats(second),
            Stats {
                power: 2,
                toughness: 2
            }
        );
        assert!(g.keywords(second).contains(Keywords::HASTE));
        g.begin_step(Step::End);
        settle(&mut g);
        assert_eq!(g.state.objects[second as usize].zone, Zone::Outside);
    }
}

#[test]
fn abandoning_copy_choices_ceases_only_the_exact_uncommitted_copy_and_keeps_future_delays() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let teach = named(&mut g, p, "Teach by Example", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: teach });
        checked(&mut g, Action::ChooseSpellCost { cost: 0 });
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        let saved = g.state.stack.last().unwrap().clone();
        settle(&mut g);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Player(p.other())]);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        let draft = g.observe(p).retarget.unwrap().item;
        assert!(draft.is_copy);
        g.remember_next_spell_copy(&saved);
        assert_eq!(g.delayed_spell_copy_view().len(), 1);
        g.abandon_copy_continuations();
        assert!(g.retarget_view().is_none());
        assert_eq!(g.state.objects[draft.card as usize].zone, Zone::Outside);
        assert!(!g.graveyard_entry_this_turn(g.object_ref(draft.card)));
        assert_eq!(g.delayed_spell_copy_view().len(), 1);
        assert!(
            g.state
                .stack
                .iter()
                .any(|s| s.card == shock && s.kind == StackKind::Spell)
        );
        g.state.pending = Pending::Priority;
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.delayed_spell_copy_view().len(), 1);
        g.expire_spell_copies();
        assert!(g.delayed_spell_copy_view().is_empty());
    }
}
