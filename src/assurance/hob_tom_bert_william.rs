//! Paid sacrifice quantities and exact-incarnation artifact resurrection.
use super::*;
const NAME: &str = "Tom, Bert, and William";
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::ChooseEntryTimestamp { .. }
            )
        });
    }
    panic!("Tom budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
    });
}
fn tom(g: &mut Game, p: Player) -> CardId {
    put(g, p, NAME, Zone::Battlefield)
}
fn activate(g: &mut Game, s: CardId, fuel: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: s,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let object = g.object_ref(fuel);
    act(g, Action::ChooseAdditionalCost { object });
    pay(g);
}
fn discard(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectDiscard { .. })
    });
    choose(g, |a| matches!(a, Action::Discard { .. }));
    finish(g);
}
fn artifact(g: &Game, s: CardId, p: Player) {
    let o = g.object_view(s);
    assert_eq!((o.zone, o.controller, o.kinds), (Zone::Battlefield, p, 16));
    assert!(!o.has_subtype("Troll"));
    assert!(o.card.is_legendary());
    assert_eq!(o.colors, 20);
    assert_eq!(o.card.activations().len(), 1);
    assert_eq!(
        g.copiable_recipe(o.reference).kinds(),
        1,
        "type effect is not copiable"
    );
    let row = g
        .observe(p)
        .animations
        .into_iter()
        .find(|r| r.object == o.reference)
        .unwrap();
    assert!(row.artifact_noncreature);
    assert_eq!(row.expires_turn, None);
    assert_eq!(
        serde_json::to_value(row).unwrap()["artifact_noncreature"],
        true
    );
}
#[test]
fn hob_tom_paid_five_five_real_murder_returns_under_death_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = put(&mut g, p, NAME, Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 1, 3];
        cast(&mut g, s, None);
        finish(&mut g);
        assert_eq!(stats(&g, s), (5, 5));
        assert!(g.has_subtype(s, "Troll"));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.change_controller(g.object_ref(s), p.other());
        let old = g.object_ref(s);
        let murder = put(&mut g, p, "Murder", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
        cast(&mut g, murder, Some(Target::Card(s)));
        finish(&mut g);
        artifact(&g, s, p.other());
        assert_eq!(g.object_view(s).owner, p);
        assert_ne!(g.object_ref(s), old);
        assert_eq!(g.object_view(murder).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn hob_tom_paid_sacrifice_current_power_tokens_and_discard_are_controller_relative() {
    for p in [Player::P0, Player::P1] {
        for power in [0, 2, 4, -1] {
            let mut g = scenario(p);
            let s = tom(&mut g, p);
            let fuel = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.change_controller(g.object_ref(fuel), p);
            g.state.objects[fuel as usize].growth.power = power - 2;
            let h = put(&mut g, p, "Shock", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            activate(&mut g, s, fuel);
            assert_eq!(g.object_view(fuel).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.state.stack.last().unwrap().sacrificed_power.as_deref(),
                Some(&power)
            );
            g.move_object(s, Zone::Exile);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectDiscard { .. })
            });
            act(&mut g, Action::Discard { card: h });
            finish(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                power.max(0) as u64
            );
            assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 0);
            assert_eq!(g.object_view(h).zone, Zone::Graveyard);
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.zone == Zone::Hand && o.owner == p)
                    .count(),
                power.max(0) as usize
            );
        }
    }
}
#[test]
fn hob_tom_sacrifice_to_exile_preserves_power_and_artifact_source_can_activate_again() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = tom(&mut g, p);
        g.move_object(s, Zone::Graveyard);
        finish(&mut g);
        artifact(&g, s, p);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        for _ in 0..2 {
            let fuel = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            activate(&mut g, s, fuel);
            assert_eq!(g.object_view(fuel).zone, Zone::Exile);
            discard(&mut g);
        }
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 4);
        assert!(!g.object_view(s).tapped);
    }
}
#[test]
fn hob_tom_legality_other_control_creature_mana_and_instant_speed() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = tom(&mut g, p);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let wrong = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let begin = Action::AnnounceActivate {
            card: s,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, begin.clone());
        let fuel = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, begin.clone());
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        g.state.active = p.other();
        g.state.step = Step::End;
        act(&mut g, begin);
        for id in [s, enemy, wrong] {
            let object = g.object_ref(id);
            reject(&mut g, Action::ChooseAdditionalCost { object });
        }
        contracts::branch_contract(g.clone());
        let object = g.object_ref(fuel);
        act(&mut g, Action::ChooseAdditionalCost { object });
        pay(&mut g);
        discard(&mut g);
    }
}
#[test]
fn hob_tom_zero_draw_empty_hand_still_resolves_and_token_fuel_uses_real_power() {
    let p = Player::P0;
    let mut g = scenario(p);
    let s = tom(&mut g, p);
    let fuel = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    g.state.objects[fuel as usize].growth.power = -2;
    activate(&mut g, s, fuel);
    finish(&mut g);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
    g.create_tokens(p, Card::from_name("@token/hob-dwarf").unwrap(), 1, false);
    finish(&mut g);
    let fuel = g
        .state
        .objects
        .iter()
        .position(|o| o.zone == Zone::Battlefield && o.is_token())
        .unwrap() as CardId;
    activate(&mut g, s, fuel);
    discard(&mut g);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
    assert_ne!(g.object_view(fuel).zone, Zone::Battlefield);
}
#[test]
fn hob_tom_preentry_artifact_replacements_and_observers_see_no_creature() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = tom(&mut g, p);
        let a = put(&mut g, p, "Elvish Archivist", Zone::Battlefield);
        put(&mut g, p, "Lifecreed Duo", Zone::Battlefield);
        put(
            &mut g,
            p.other(),
            "Authority of the Consuls",
            Zone::Battlefield,
        );
        g.move_object(s, Zone::Graveyard);
        finish(&mut g);
        artifact(&g, s, p);
        assert!(!g.object_view(s).tapped);
        assert_eq!(g.counter_count(a, CounterKind::PlusOne), 2);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 20);
    }
}
#[test]
fn hob_tom_exact_graveyard_incarnation_stale_countered_and_replaced_deaths() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..4 {
            let mut g = scenario(p);
            let s = tom(&mut g, p);
            if mode == 3 {
                put(&mut g, p, "Rest in Peace", Zone::Battlefield);
            }
            g.move_object(s, Zone::Graveyard);
            if mode == 0 {
                g.move_object(s, Zone::Exile);
                g.move_object(s, Zone::Graveyard);
            }
            if mode == 1 {
                g.prepare_triggers();
                until(&mut g, |g| !g.state.stack.is_empty());
                let id = g.state.stack.last().unwrap().id;
                assert!(g.counter_stack_item(id, false));
            }
            if mode == 2 {
                g.move_object(s, Zone::Battlefield);
                g.move_object(s, Zone::Graveyard);
            }
            finish(&mut g);
            match mode {
                0 | 1 => assert_eq!(g.object_view(s).zone, Zone::Graveyard),
                2 => artifact(&g, s, p),
                _ => assert_eq!(g.object_view(s).zone, Zone::Exile),
            }
        }
    }
}
#[test]
fn hob_tom_noncreature_death_does_not_return_but_later_animation_can_die_again() {
    use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec};
    let p = Player::P0;
    let mut g = scenario(p);
    let s = tom(&mut g, p);
    g.move_object(s, Zone::Graveyard);
    finish(&mut g);
    artifact(&g, s, p);
    let saved = g.snapshot();
    g.move_object(s, Zone::Graveyard);
    finish(&mut g);
    assert_eq!(g.object_view(s).zone, Zone::Graveyard);
    g.restore(&saved).unwrap();
    g.change_object_form(
        g.object_ref(s),
        AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((3, 3)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        },
        characteristic_layers::AnimationTypeMode::AddCreature,
    );
    assert!(g.is_creature(s));
    g.move_object(s, Zone::Graveyard);
    finish(&mut g);
    artifact(&g, s, p);
    assert_eq!(
        g.observe(p)
            .animations
            .iter()
            .filter(|r| r.object == g.object_ref(s))
            .count(),
        1
    );
}
#[test]
fn hob_tom_token_copy_dies_without_return_and_ability_loss_suppresses_trigger() {
    let p = Player::P0;
    for token in [true, false] {
        let mut g = scenario(p);
        let s = tom(&mut g, p);
        if token {
            let source = g.object_view(s);
            let recipe = g.copiable_recipe(source.reference);
            g.move_object(s, Zone::Outside);
            g.stage_copy_creation(p, recipe, 1);
            let t = (0..g.state.objects.len())
                .find(|&i| g.state.objects[i].is_token())
                .unwrap() as CardId;
            g.move_object(t, Zone::Graveyard);
            finish(&mut g);
            assert_ne!(g.object_view(t).zone, Zone::Battlefield);
        } else {
            let aura = put(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
            g.state.objects[aura as usize].attached_to = Some(g.object_ref(s));
            assert_eq!(stats(&g, s), (1, 1));
            g.move_object(s, Zone::Graveyard);
            finish(&mut g);
            assert_eq!(g.object_view(s).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn hob_tom_staff_doubles_ordinary_death_trigger_but_only_first_exact_return_succeeds() {
    let p = Player::P0;
    let mut g = scenario(p);
    let s = tom(&mut g, p);
    let staff = put(&mut g, p, "Wizard's Staff", Zone::Battlefield);
    g.state.objects[staff as usize].attached_to = Some(g.object_ref(s));
    assert_eq!(g.object_view(s).additional_triggers, 1);
    g.move_object(s, Zone::Graveyard);
    g.prepare_triggers();
    assert_eq!(
        g.state.pending_triggers.len() + g.state.ordering_triggers.len() + g.state.stack.len(),
        2
    );
    finish(&mut g);
    artifact(&g, s, p);
    assert_eq!(g.object_ref(s).generation, 2);
    assert_eq!(g.observe(p).animations.len(), 1);
}
#[test]
fn hob_tom_private_discard_owned_raw_index_and_restore_retains_paid_quantity() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = tom(&mut g, p);
        let f = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        activate(&mut g, s, f);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectDiscard { .. })
        });
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        for (id, o) in g
            .state
            .objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.zone == Zone::Hand && o.owner == p)
        {
            assert_eq!(o.owner, p);
            assert!(g.observe(p).objects.iter().any(|o| o.id == id as CardId));
            assert!(
                g.observe(p.other())
                    .objects
                    .iter()
                    .all(|o| o.id != id as CardId)
            );
        }
        let saved = g.snapshot();
        let actor = g.actor().unwrap();
        let rev = g.revision();
        let actions = g.legal_actions();
        let i = actions
            .iter()
            .position(|a| matches!(a, Action::Discard { .. }))
            .unwrap();
        let mut indexed = g.clone();
        let mut raw = g.clone();
        drop(g);
        indexed.apply_index(actor, rev, i).unwrap();
        raw.apply(actor, actions[i].clone()).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        finish(&mut raw);
        finish(&mut indexed);
        let expected = raw.snapshot();
        assert_eq!(indexed.snapshot(), expected);
        raw.restore(&saved).unwrap();
        act(&mut raw, actions[i].clone());
        finish(&mut raw);
        assert_eq!(raw.snapshot(), expected);
    }
}
#[test]
fn hob_tom_miracle_pause_preserves_power_after_source_and_fuel_departures() {
    let p = Player::P0;
    let mut g = scenario(p);
    let s = tom(&mut g, p);
    put(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
    let old = g.state.players[p.index()].library.clone();
    for id in old {
        g.move_object(id, Zone::Outside);
    }
    for _ in 0..4 {
        put(&mut g, p, "Shock", Zone::Library);
    }
    let fuel = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    activate(&mut g, s, fuel);
    until(&mut g, |g| {
        matches!(g.state.pending, Pending::MiracleReveal { .. })
    });
    g.move_object(s, Zone::Exile);
    g.move_object(fuel, Zone::Battlefield);
    assert!(g.observe(p).miracle.is_some());
    assert!(g.observe(p.other()).miracle.is_none());
    contracts::branch_contract(g.clone());
    act(&mut g, Action::DeclineMiracle);
    discard(&mut g);
    assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
}
#[test]
fn hob_tom_form_permanent_cleanup_blink_and_other_types_are_preserved() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = tom(&mut g, p);
        g.move_object(s, Zone::Graveyard);
        finish(&mut g);
        for _ in 0..2 {
            g.state.step = Step::Cleanup;
            g.finish_cleanup();
            finish(&mut g);
            artifact(&g, s, p);
        }
        g.move_object(s, Zone::Exile);
        g.move_object(s, Zone::Battlefield);
        assert_eq!(g.object_view(s).kinds, 1);
        assert!(g.has_subtype(s, "Troll"));
        assert!(g.observe(p).animations.is_empty());
        // Generic form contract: unrelated card types/subtypes and printed abilities survive.
        let pick = put(&mut g, p, "Goldvein Pick", Zone::Graveyard);
        let mut m = g.entry_move(pick, Zone::Battlefield, false);
        m.artifact_return = true;
        assert_eq!(m.entry_kinds(), 16);
        g.stage_entries(vec![m], None);
        finish(&mut g);
        assert_eq!(g.object_view(pick).kinds, 16);
        assert!(g.has_subtype(pick, "Equipment"));
        assert_eq!(g.object_view(pick).card.activations().len(), 1);
    }
}
#[test]
fn hob_tom_suspended_generic_artifact_entry_owned_restore_and_prevented_move() {
    // No admitted effect prevents this graveyard return. Exercise the pipeline's
    // skipped-move result directly, without inventing a printed prevention card.
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let s = put(&mut g, p, "Adaptive Automaton", Zone::Graveyard);
        let mut m = g.entry_move(s, Zone::Battlefield, false);
        m.artifact_return = true;
        g.stage_entries(vec![m], None);
        assert!(g.advance_entries());
        assert!(matches!(g.state.pending, Pending::EntryValue { .. }));
        assert!(g.state.entry_batch.as_ref().unwrap().moves[0].artifact_return);
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut child = g.clone();
        drop(g);
        choose(&mut child, |a| matches!(a, Action::ChooseEntryValue { .. }));
        finish(&mut child);
        assert_eq!(child.object_view(s).kinds, 16);
        assert!(!child.has_subtype(s, "Construct"));
        let expected = child.snapshot();
        child.restore(&saved).unwrap();
        choose(&mut child, |a| matches!(a, Action::ChooseEntryValue { .. }));
        finish(&mut child);
        assert_eq!(child.snapshot(), expected);
        child.restore(&saved).unwrap();
        child.state.entry_batch.as_mut().unwrap().moves[0].skipped = true;
        choose(&mut child, |a| matches!(a, Action::ChooseEntryValue { .. }));
        finish(&mut child);
        assert_eq!(child.object_view(s).zone, Zone::Graveyard);
        assert!(child.observe(p).animations.is_empty());
    }
}
#[test]
fn hob_tom_copied_other_physical_types_return_as_artifact_with_correlated_subtypes() {
    use crate::cards::{AnimationCreatureTypes, AnimationDuration, AnimationSpec};
    for p in [Player::P0, Player::P1] {
        for (name, retained, removed) in [
            ("Goldvein Pick", Some("Equipment"), None),
            ("Forest", None, Some("Forest")),
            ("Witness Protection", None, Some("Aura")),
        ] {
            let mut g = scenario(p);
            let s = put(&mut g, p, name, Zone::Battlefield);
            g.apply_temporary_copy(
                g.object_ref(s),
                CopyRecipe::new(Card::from_name(NAME).unwrap()),
                CopyDeadline::EndOfTurn,
            );
            assert_eq!(stats(&g, s), (5, 5));
            assert!(g.has_subtype(s, "Troll"));
            g.move_object(s, Zone::Graveyard);
            finish(&mut g);
            assert_eq!(g.object_view(s).zone, Zone::Battlefield);
            assert_eq!(g.object_view(s).kinds, 16);
            assert_eq!(g.object_view(s).name(), name);
            assert!(!g.has_subtype(s, "Troll"));
            if let Some(t) = retained {
                assert!(g.has_subtype(s, t));
            }
            if let Some(t) = removed {
                assert!(!g.has_subtype(s, t));
            }
            let v = serde_json::to_value(g.object_view(s).granted_subtypes).unwrap();
            assert_eq!(v["card_types"], 16);
            g.change_object_form(
                g.object_ref(s),
                AnimationSpec {
                    artifact: false,
                    creature_types: AnimationCreatureTypes::Named(&["Bear"]),
                    base: Some((3, 3)),
                    keywords: Keywords::NONE,
                    duration: AnimationDuration::EndOfTurn,
                },
                characteristic_layers::AnimationTypeMode::AddCreature,
            );
            assert_eq!(g.object_view(s).kinds, 17);
            assert!(g.has_subtype(s, "Bear"));
            if let Some(t) = retained {
                assert!(g.has_subtype(s, t));
            }
            if let Some(t) = removed {
                assert!(!g.has_subtype(s, t));
            }
        }
        let mut g = scenario(p);
        let s = tom(&mut g, p);
        g.move_object(s, Zone::Graveyard);
        finish(&mut g);
        g.change_object_form(
            g.object_ref(s),
            AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Unchanged,
                base: Some((3, 3)),
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            },
            characteristic_layers::AnimationTypeMode::AddCreature,
        );
        assert!(
            g.has_subtype(s, "Troll"),
            "later creature type restores correlated printed creature subtype"
        );
    }
}
#[test]
fn hob_tom_late_draw_capacity_failure_rolls_back_action_but_keeps_prior_paid_cost() {
    let p = Player::P0;
    let mut g = scenario(p);
    let s = tom(&mut g, p);
    let fuel = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
    activate(&mut g, s, fuel);
    g.state.players[p.index()].cards_drawn_this_turn = u64::MAX;
    act(&mut g, Action::Pass);
    let before = g.snapshot();
    let rev = g.revision();
    let actor = g.actor().unwrap();
    assert!(matches!(
        g.apply(actor, Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
    assert_eq!(g.object_view(fuel).zone, Zone::Graveyard);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(
        g.state.stack.last().unwrap().sacrificed_power.as_deref(),
        Some(&2)
    );
}
