//! Nonlegendary copiable values, nontoken recursion guard and current subtype mana.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::ChooseReplacement { .. }
            )
        });
    }
    panic!("Notary choice budget: {:?}", g.state.pending);
}
fn drive(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
    });
}
fn card() -> Card {
    Card::from_name("The Notary Hobbits").unwrap()
}
fn paid_entry(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "The Notary Hobbits", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 3];
    cast(g, id, None);
    until(g, |g| g.on_battlefield(id));
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn tokens(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.controller == p && o.is_token())
                .then_some(id as CardId)
        })
        .collect()
}
fn source(g: &mut Game, p: Player) -> CardId {
    put(g, p, "The Notary Hobbits", Zone::Battlefield)
}
fn types(g: &mut Game, id: CardId, names: &'static [&'static str]) {
    let recipient = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types: names,
            additional: false,
            timestamp,
        });
}
fn mana(id: CardId, n: u32) -> Action {
    Action::ActivateMana {
        card: id,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: [0, 0, 0, 0, 0, n],
    }
}
fn age(g: &mut Game, p: Player) {
    for o in &mut g.state.objects {
        if o.zone == Zone::Battlefield && o.controller == p {
            o.summoning_sick = false;
        }
    }
}
#[test]
fn hob_notary_paid_entry_copies_exact_printed_values_not_counters_pump_tap_or_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = paid_entry(&mut g, p);
        g.add_counter(id, CounterKind::PlusOne, 2);
        g.state.objects[id as usize].tapped = true;
        g.state.objects[id as usize].damage = 1;
        let spell = put(&mut g, p, "Giant Growth", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        cast(&mut g, spell, Some(Target::Card(id)));
        drive(&mut g);
        assert_eq!(stats(&g, id), (6, 6));
        assert!(g.object_view(id).is_legendary());
        let copies = tokens(&g, p);
        assert_eq!(copies.len(), 2);
        for copy in copies {
            let view = g.object_view(copy);
            assert_eq!(
                (view.power, view.toughness, view.colors, view.kinds),
                (1, 1, 16, 1)
            );
            assert!(!view.is_legendary() && !view.tapped && view.summoning_sick);
            assert!(view.has_subtype("Halfling") && view.has_subtype("Advisor"));
            assert_eq!(view.card.fixed_mana_value(), 5);
            assert_eq!(view.name(), "The Notary Hobbits");
            assert_eq!(view.owner, p);
            assert_eq!(view.counters.get(CounterKind::PlusOne), 0);
            assert_eq!(g.state.objects[copy as usize].damage, 0);
            assert_eq!(view.card.activations().len(), 1);
        }
        assert!(g.state.pending_triggers.is_empty());
        assert!(tokens(&g, p.other()).is_empty());
    }
}
#[test]
fn hob_notary_token_etb_has_no_recursion_and_copy_of_nonlegendary_copy_keeps_exception() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        paid_entry(&mut g, p);
        drive(&mut g);
        let first = tokens(&g, p)[0];
        let recipe = g.copiable_recipe(g.object_ref(first));
        assert_eq!(recipe.overrides.unwrap().legendary, Some(false));
        g.stage_copy_creation(p, recipe, 1);
        g.give_priority();
        g.prepare_triggers();
        drive(&mut g);
        assert_eq!(tokens(&g, p).len(), 3);
        assert!(
            tokens(&g, p)
                .into_iter()
                .all(|id| !g.object_view(id).is_legendary())
        );
        let mut g = scenario(p);
        g.stage_copy_creation(p, CopyRecipe::new(card()), 1);
        g.give_priority();
        g.prepare_triggers();
        drive(&mut g);
        assert_eq!(
            tokens(&g, p).len(),
            1,
            "even a legendary Notary token never copies itself"
        );
    }
}
#[test]
fn hob_notary_nontoken_copy_entry_preserves_other_copiable_exceptions() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = put(&mut g, p, "Bear Cub", Zone::Hand);
        let events = g.move_objects_raw(&[id], Zone::Battlefield);
        let mut recipe = CopyRecipe::new(card()).with_exceptions(CopyExceptions {
            artifact: true,
            base: Some((3, 4)),
            ..CopyExceptions::NONE
        });
        recipe.overrides = Some(CopyOverrides {
            name: Some("Notary blueprint"),
            legendary: Some(true),
            keywords: Keywords::REACH,
            ..CopyOverrides::NONE
        });
        g.apply_temporary_copy(g.object_ref(id), recipe, CopyDeadline::EndOfTurn);
        g.finish_zone_events(&events, &[]);
        g.give_priority();
        g.prepare_triggers();
        drive(&mut g);
        assert_eq!(tokens(&g, p).len(), 2);
        for copy in tokens(&g, p) {
            let v = g.object_view(copy);
            assert_eq!((v.power, v.toughness, v.kinds, v.colors), (3, 4, 17, 16));
            assert_eq!(v.name(), "Notary blueprint");
            assert!(v.has_subtype("Halfling") && v.has_subtype("Advisor"));
            assert!(!v.is_legendary());
            assert!(v.keywords.contains(Keywords::REACH));
            let copied = g.copiable_recipe(g.object_ref(copy));
            assert_eq!(copied.exceptions, recipe.exceptions);
            assert_eq!(copied.overrides.unwrap().legendary, Some(false));
            assert_eq!(
                copied.overrides.unwrap().name,
                recipe.overrides.unwrap().name
            );
        }
    }
}
#[test]
fn hob_notary_resolution_copies_current_blueprint_or_exact_departed_source_lki() {
    for p in [Player::P0, Player::P1] {
        for left in [false, true] {
            let mut g = scenario(p);
            let id = paid_entry(&mut g, p);
            g.apply_temporary_copy(
                g.object_ref(id),
                CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
                CopyDeadline::EndOfTurn,
            );
            if left {
                g.move_object(id, Zone::Exile);
            }
            drive(&mut g);
            assert_eq!(tokens(&g, p).len(), 2);
            for copy in tokens(&g, p) {
                assert_eq!(g.object_view(copy).name(), "Bear Cub");
                assert_eq!(stats(&g, copy), (2, 2));
                assert!(!g.object_view(copy).is_legendary());
            }
        }
        let mut g = scenario(p);
        let id = paid_entry(&mut g, p);
        let before = g.object_ref(id);
        g.move_object(id, Zone::Exile);
        g.move_object(id, Zone::Battlefield);
        assert_ne!(g.object_ref(id), before);
        g.apply_temporary_copy(
            g.object_ref(id),
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        g.give_priority();
        g.prepare_triggers();
        drive(&mut g);
        let all = tokens(&g, p);
        assert_eq!(
            all.iter()
                .filter(|&&id| g.object_view(id).name() == "The Notary Hobbits")
                .count(),
            2,
            "old incarnation uses its Notary LKI"
        );
        assert_eq!(
            all.iter()
                .filter(|&&id| g.object_view(id).name() == "Bear Cub")
                .count(),
            2,
            "new incarnation's trigger uses current Bear values"
        );
    }
}
#[test]
fn hob_notary_copy_tokens_belong_to_frozen_ability_controller_after_source_loss_or_transfer() {
    for p in [Player::P0, Player::P1] {
        for gone in [false, true] {
            let mut g = scenario(p);
            let id = paid_entry(&mut g, p);
            if gone {
                g.move_object(id, Zone::Graveyard);
            } else {
                g.change_controller(g.object_ref(id), p.other());
            }
            drive(&mut g);
            assert_eq!(tokens(&g, p).len(), 2);
            assert!(tokens(&g, p.other()).is_empty());
            assert!(
                tokens(&g, p)
                    .into_iter()
                    .all(|id| g.object_view(id).owner == p)
            );
        }
    }
}
#[test]
fn hob_notary_trigger_multiplication_and_token_replacement_multiply_without_recursion() {
    for p in [Player::P0, Player::P1] {
        for doubled in [false, true] {
            let mut g = scenario(p);
            put(&mut g, p, "Bifur, Melodic Rider", Zone::Battlefield);
            put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
            if doubled {
                put(&mut g, p, "Doubling Season", Zone::Battlefield);
            }
            let id = put(&mut g, p, "The Notary Hobbits", Zone::Hand);
            let events = g.move_objects_raw(&[id], Zone::Battlefield);
            types(&mut g, id, &["Dwarf"]);
            g.finish_zone_events(&events, &[]);
            g.give_priority();
            g.prepare_triggers();
            drive(&mut g);
            assert!(g.state.players[p.index()].enduring_story);
            assert_eq!(tokens(&g, p).len(), if doubled { 8 } else { 4 });
            assert!(
                tokens(&g, p)
                    .into_iter()
                    .all(|id| g.has_subtype(id, "Halfling") && !g.has_subtype(id, "Dwarf")),
                "layer4 subtype overlay is not copiable"
            );
        }
    }
}
#[test]
fn hob_notary_two_replacements_suspend_owned_copy_recipe_with_honest_public_views() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let secret = put(&mut g, p, "Shock", Zone::Hand);
        paid_entry(&mut g, p);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Replacement { .. })
        });
        assert!(g.observe(p.other()).objects.iter().all(|v| v.id != secret));
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        let actions = raw.legal_actions();
        let i = actions
            .iter()
            .position(|a| matches!(a, Action::ChooseReplacement { .. }))
            .unwrap();
        let a = actions[i].clone();
        raw.apply(p, a.clone()).unwrap();
        indexed.apply_index(p, indexed.revision(), i).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        drive(&mut raw);
        drive(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(tokens(&raw, p).len(), 8);
        indexed.restore(&saved).unwrap();
        indexed.apply(p, a).unwrap();
        drive(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}
#[test]
fn hob_notary_actual_mana_current_control_subtypes_zero_and_copied_ability() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = paid_entry(&mut g, p);
        drive(&mut g);
        let copies = tokens(&g, p);
        reject(&mut g, mana(id, 3));
        reject(&mut g, mana(copies[0], 3));
        age(&mut g, p);
        source(&mut g, p.other());
        let added = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        types(&mut g, added, &["Halfling"]);
        types(&mut g, copies[0], &["Bear"]);
        g.change_controller(g.object_ref(copies[1]), p.other());
        reject(&mut g, mana(id, 3));
        act(&mut g, mana(id, 2));
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 2]);
        assert!(g.state.stack.is_empty());
        assert!(g.object_view(id).tapped);
        reject(&mut g, mana(id, 2));
        let mut g = scenario(p);
        let id = source(&mut g, p);
        types(&mut g, id, &["Bear"]);
        g.state.players[p.index()].mana = [0; 6];
        assert!(g.legal_actions().contains(&mana(id, 0)));
        act(&mut g, mana(id, 0));
        assert!(g.object_view(id).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        let mut g = scenario(p);
        paid_entry(&mut g, p);
        drive(&mut g);
        age(&mut g, p);
        let copy = tokens(&g, p)[0];
        act(&mut g, mana(copy, 3));
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 3]);
        assert!(g.object_view(copy).tapped);
    }
}
#[test]
fn hob_notary_mana_funds_generic_but_not_colored_cost_and_cost_root_is_owned() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = paid_entry(&mut g, p);
        drive(&mut g);
        age(&mut g, p);
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Hand);
        let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: growth });
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: pick })
        );
        act(&mut g, Action::AnnounceCast { card: pick });
        contracts::branch_contract(g.clone());
        act(&mut g, mana(id, 3));
        pay(&mut g);
        drive(&mut g);
        assert_eq!(g.object_view(pick).zone, Zone::Battlefield);
        assert!(g.object_view(id).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 1]);
    }
}
#[test]
fn hob_notary_latest_departure_keeps_changed_copy_exceptions_separate_across_reentry() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = paid_entry(&mut g, p);
        let mut recipe = CopyRecipe::new(card()).with_exceptions(CopyExceptions {
            artifact: true,
            base: Some((3, 4)),
            ..CopyExceptions::NONE
        });
        recipe.overrides = Some(CopyOverrides {
            name: Some("Departing Notary"),
            legendary: Some(true),
            keywords: Keywords::REACH,
            ..CopyOverrides::NONE
        });
        g.apply_temporary_copy(g.object_ref(id), recipe, CopyDeadline::EndOfTurn);
        g.move_object(id, Zone::Exile);
        g.move_object(id, Zone::Battlefield);
        g.apply_temporary_copy(
            g.object_ref(id),
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        g.move_object(id, Zone::Graveyard);
        g.give_priority();
        g.prepare_triggers();
        drive(&mut g);
        let all = tokens(&g, p);
        assert_eq!(all.len(), 4);
        let exceptional: Vec<_> = all
            .iter()
            .copied()
            .filter(|&id| g.object_view(id).name() == "Departing Notary")
            .collect();
        assert_eq!(exceptional.len(), 2);
        for copy in exceptional {
            let v = g.object_view(copy);
            assert_eq!((v.power, v.toughness, v.kinds), (3, 4, 17));
            assert!(v.keywords.contains(Keywords::REACH));
            assert!(!v.is_legendary());
        }
        assert_eq!(
            all.iter()
                .filter(|&&id| g.object_view(id).name() == "Bear Cub" && stats(&g, id) == (2, 2))
                .count(),
            2
        );
    }
}
