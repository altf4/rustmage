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
        let reference = g.object_ref(token);
        g.change_controller(reference, p);
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
        let witness = named(&mut g, p, "Witness Protection", Zone::Battlefield);
        g.attach_equipment(g.object_ref(witness), g.object_ref(live));
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
