//! Whole-card paid scenarios supplement the separately reviewed counter core.
//! Explicit step, draw-event and packet-copy probes are bounded shared-executor
//! fixtures, not claims of natural Standard games or new copy-card admissions.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{cast, checked, settle, setup};
use super::msh_counter_quantity_core_tests::named;
use super::*;
const DOC: &str = "Doc Samson, Super Psychiatrist";
const HYDE: &str = "Mister Hyde, Monster Within";
const ANT: &str = "The Astonishing Ant-Man";
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    let bytes = g.state_bytes();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
    assert_eq!(g.state_bytes(), bytes);
}
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut observation = g.observe(p);
        observation.revision = 0;
        serde_json::to_value(observation).unwrap()
    })
}
fn orphan(g: Game, action: Action) -> Game {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let before = g.snapshot();
    let before_views = views(&g);
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, action.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    let after = raw.snapshot();
    assert_eq!(indexed.snapshot(), after);
    assert_eq!(views(&raw), views(&indexed));
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), before_views);
    assert_eq!(
        raw.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, action).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
fn mana_for(g: &mut Game, p: Player, card: CardId) {
    let (mut colored, generic) = g.object_view(card).card.cost();
    colored[5] += generic;
    g.state.players[p.index()].mana = colored;
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    mana_for(g, p, id);
    cast(g, id, None, targets);
    settle(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..100 {
        if stop(g) {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
                )
            })
            .unwrap_or_else(|| panic!("unhandled {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("stalled {:?}", g.state.pending)
}
fn upkeep(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.begin_step(Step::Upkeep);
    until(g, |g| matches!(g.state.pending, Pending::Modes { .. }));
}
fn pending(g: &Game, source: CardId) -> usize {
    g.state
        .pending_triggers
        .iter()
        .filter(|s| s.card == source)
        .count()
}
fn plus(g: &Game, id: CardId) -> u64 {
    g.counter_count(id, CounterKind::PlusOne)
}
fn copy_source(g: &mut Game, p: Player, id: CardId) -> CardId {
    let definition = g.object_view(id).card;
    // Production token-copy entry, with its own incarnation and provider indexes.
    let copy = g.create_copy_groups_inner(
        p,
        &[TokenGroupView {
            token: definition,
            amount: 1,
            tapped: false,
        }],
        &[Some(CopyRecipe::new(definition))],
        None,
    )[0]
    .card;
    g.advance_entries();
    settle(g);
    copy
}
fn activate(id: CardId) -> Action {
    Action::AnnounceActivate {
        card: id,
        ability: 0,
        counter_payment: Counters::default(),
    }
}
fn pay(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    checked(g, action);
}
fn insects(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card == Card::from_name("@token/3512fba4bfa3f3fcf0f1eb9a").unwrap())
            .then_some(i as CardId)
        })
        .collect()
}
#[test]
fn msh_counter_three_real_costs_reject_shortage_wrong_color_and_sick_tap() {
    for p in [Player::P0, Player::P1] {
        for name in [DOC, HYDE, ANT] {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            reject(&mut g, Action::AnnounceCast { card: id });
            let (_, generic) = g.object_view(id).card.cost();
            g.state.players[p.index()].mana[5] = generic + 2;
            reject(&mut g, Action::AnnounceCast { card: id });
            mana_for(&mut g, p, id);
            cast(&mut g, id, None, &[]);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            if name == ANT {
                g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 2];
                reject(&mut g, activate(id));
            }
            assert!(g.object_view(id).is_legendary());
        }
    }
}
#[test]
fn msh_counter_three_each_real_legendary_duplicate_uses_owned_legend_rule() {
    for p in [Player::P0, Player::P1] {
        for name in [DOC, HYDE, ANT] {
            let mut g = setup(p);
            let old = paid(&mut g, p, name, &[]);
            let new = named(&mut g, p, name, Zone::Hand);
            mana_for(&mut g, p, new);
            cast(&mut g, new, None, &[]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::Legend { .. })
            });
            let keep = g.object_ref(new);
            g = orphan(g, Action::KeepLegend { object: keep });
            settle(&mut g);
            assert_eq!(g.object_view(old).zone, Zone::Graveyard);
            assert_eq!(g.object_view(new).zone, Zone::Battlefield);
        }
    }
}
#[test]
fn msh_counter_three_hyde_only_own_upkeep_and_one_mode_with_no_targets() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, HYDE, &[]);
        for (active, step) in [
            (p.other(), Step::Upkeep),
            (p, Step::Draw),
            (p, Step::Main1),
            (p, Step::End),
        ] {
            g.state.active = active;
            g.begin_step(step);
            assert_eq!(pending(&g, h), 0);
        }
        upkeep(&mut g, p);
        reject(&mut g, Action::ChooseMode { mode: 2 });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(h),
            },
        );
        g = orphan(g, Action::ChooseMode { mode: 0 });
        assert!(
            g.state
                .stack
                .iter()
                .any(|s| s.card == h && s.targets.is_empty())
        );
        settle(&mut g);
        assert_eq!(plus(&g, h), 1);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        g.state.turn += 2;
        upkeep(&mut g, p);
        checked(&mut g, Action::ChooseMode { mode: 0 });
        settle(&mut g);
        assert_eq!(plus(&g, h), 2);
    }
}
#[test]
fn msh_counter_three_hyde_first_mode_tracks_exact_source_after_control_departure_and_blink() {
    for p in [Player::P0, Player::P1] {
        for change in 0..3 {
            let mut g = setup(p);
            let h = paid(&mut g, p, HYDE, &[]);
            upkeep(&mut g, p);
            checked(&mut g, Action::ChooseMode { mode: 0 });
            let before = g.object_ref(h);
            // Production transition probe after announcement, before resolution.
            match change {
                0 => g.change_controller(before, p.other()),
                1 => {
                    g.move_object(h, Zone::Graveyard);
                }
                _ => {
                    g.move_object(h, Zone::Exile);
                    g.move_object(h, Zone::Battlefield);
                }
            }
            settle(&mut g);
            assert_eq!(plus(&g, h), u64::from(change == 0));
            if change == 2 {
                assert_ne!(g.object_ref(h), before);
            }
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        }
    }
}
#[test]
fn msh_counter_three_hyde_upkeep_uses_current_control_and_suppression_at_event() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, HYDE, &[]);
        g.change_controller(g.object_ref(h), p.other());
        g.state.active = p;
        g.begin_step(Step::Upkeep);
        assert_eq!(pending(&g, h), 0);
        upkeep(&mut g, p.other());
        assert_eq!(g.actor(), Some(p.other()));
        checked(&mut g, Action::ChooseMode { mode: 0 });
        settle(&mut g);
        assert_eq!(plus(&g, h), 1);
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(h)]);
        g.state.active = p.other();
        g.begin_step(Step::Upkeep);
        assert_eq!(pending(&g, h), 0);
        g.move_object(aura, Zone::Graveyard);
        upkeep(&mut g, p.other());
        checked(&mut g, Action::ChooseMode { mode: 0 });
        settle(&mut g);
        assert_eq!(plus(&g, h), 2);
    }
}
#[test]
fn msh_counter_three_ant_paid_multi_draw_triggers_for_every_own_card_not_opponents() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = paid(&mut g, p, ANT, &[]);
        for expected in [2, 4] {
            paid(&mut g, p, "Divination", &[]);
            assert_eq!(plus(&g, a), expected);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, expected);
        }
        // Same production draw-event boundary, independent of active turn.
        g.draw_many(p.other(), 2);
        assert_eq!(pending(&g, a), 0);
        settle(&mut g);
        assert_eq!(plus(&g, a), 4);
        assert_eq!(g.state.players[p.other().index()].cards_drawn_this_turn, 2);
    }
}
#[test]
fn msh_counter_three_ant_normal_draw_step_queues_exact_event_with_private_drawn_identity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = paid(&mut g, p, ANT, &[]);
        let top = named(&mut g, p, "Shock", Zone::Library);
        g.begin_step(Step::Upkeep);
        checked(&mut g, Action::Pass);
        checked(&mut g, Action::Pass);
        assert_eq!(g.state.step, Step::Draw);
        assert_eq!(g.object_view(top).zone, Zone::Hand);
        let packet = g
            .state
            .pending_triggers
            .iter()
            .find(|s| s.card == a)
            .or_else(|| g.state.ordering_triggers.iter().find(|s| s.card == a))
            .or_else(|| g.state.stack.iter().find(|s| s.card == a))
            .unwrap();
        assert_eq!(
            packet.draw_event,
            Some(DrawTriggerView {
                player: p,
                ordinal: 1
            })
        );
        assert_eq!(packet.object, g.object_ref(a));
        let mut hidden = g.clone();
        hidden.state.objects[top as usize].card = Card::from_name("Forest").unwrap();
        assert_eq!(
            views(&g)[p.other().index()],
            views(&hidden)[p.other().index()]
        );
        assert_eq!(g.legal_actions(), hidden.legal_actions());
        settle(&mut g);
        assert_eq!(plus(&g, a), 1);
    }
}
#[test]
fn msh_counter_three_ant_hand_and_graveyard_moves_are_not_draw_and_failed_draw_has_no_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = paid(&mut g, p, ANT, &[]);
        let card = named(&mut g, p, "Shock", Zone::Hand);
        assert_eq!(pending(&g, a), 0);
        g.move_object(card, Zone::Graveyard);
        let top = *g.state.players[p.index()].library.last().unwrap();
        g.move_object(top, Zone::Graveyard);
        assert_eq!(pending(&g, a), 0);
        for id in g.state.players[p.index()].library.clone() {
            g.move_object(id, Zone::Exile);
        }
        // A paid draw spell attempts an empty library draw; no successful draw event.
        paid(&mut g, p, "Divination", &[]);
        assert_eq!(plus(&g, a), 0);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        assert_eq!(g.outcome(), Some(Outcome::Winner(p.other())));
    }
}
#[test]
fn msh_counter_three_ant_draw_packets_keep_exact_source_through_control_departure_blink() {
    for p in [Player::P0, Player::P1] {
        for change in 0..3 {
            let mut g = setup(p);
            let a = paid(&mut g, p, ANT, &[]);
            let div = named(&mut g, p, "Divination", Zone::Hand);
            mana_for(&mut g, p, div);
            cast(&mut g, div, None, &[]);
            until(&mut g, |g| {
                pending(g, a) == 2
                    || g.state
                        .ordering_triggers
                        .iter()
                        .filter(|s| s.card == a)
                        .count()
                        == 2
            });
            let old = g.object_ref(a);
            match change {
                0 => g.change_controller(old, p.other()),
                1 => {
                    g.move_object(a, Zone::Graveyard);
                }
                _ => {
                    g.move_object(a, Zone::Exile);
                    g.move_object(a, Zone::Battlefield);
                }
            }
            settle(&mut g);
            assert_eq!(plus(&g, a), if change == 0 { 2 } else { 0 });
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
            if change == 2 {
                assert_ne!(g.object_ref(a), old);
            }
        }
    }
}
#[test]
fn msh_counter_three_ant_current_controller_receives_draws_and_suppression_restores_observer() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = paid(&mut g, p, ANT, &[]);
        g.change_controller(g.object_ref(a), p.other());
        g.draw_many(p, 1);
        assert_eq!(pending(&g, a), 0);
        g.draw_many(p.other(), 1);
        settle(&mut g);
        assert_eq!(plus(&g, a), 1);
        let aura = paid(&mut g, p, "Witness Protection", &[Target::Card(a)]);
        g.draw_many(p.other(), 2);
        assert_eq!(pending(&g, a), 0);
        settle(&mut g);
        assert_eq!(plus(&g, a), 1);
        g.move_object(aura, Zone::Graveyard);
        g.draw_many(p.other(), 2);
        settle(&mut g);
        assert_eq!(plus(&g, a), 3);
    }
}
#[test]
fn msh_counter_three_ant_draw_trigger_after_source_loses_ability_still_resolves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = paid(&mut g, p, ANT, &[]);
        // A draw-event executor fixture freezes a real queued source packet.
        g.draw_many(p, 1);
        let trigger = g
            .state
            .pending_triggers
            .iter()
            .find(|s| s.card == a)
            .unwrap()
            .id;
        g.prepare_triggers();
        assert!(g.state.stack.iter().any(|s| s.id == trigger));
        // Flash permits an ordinary paid Aura response after the trigger.
        paid(&mut g, p, "Eaten by Piranhas", &[Target::Card(a)]);
        assert_eq!(plus(&g, a), 1);
        assert_eq!(g.stats(a).power, 2);
        g.draw_many(p, 1);
        assert_eq!(pending(&g, a), 0);
    }
}
#[test]
fn msh_counter_three_ant_copied_paid_activation_keeps_removed_quantity_without_new_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = paid(&mut g, p, ANT, &[]);
        g.state.objects[a as usize].summoning_sick = false;
        g.set_counters(a, Counters::of(CounterKind::PlusOne, 3));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 2];
        checked(&mut g, activate(a));
        g = orphan(g, Action::ChooseSourceCounterAmount { amount: 2 });
        pay(&mut g);
        assert_eq!(plus(&g, a), 1);
        let mut copy = g.state.stack.last().unwrap().clone();
        copy.id = g.next_stack_id();
        copy.is_copy = true;
        // Bounded packet-copy fixture, not a new ability-copy producer admission.
        g.state.stack.push(copy);
        g.move_object(a, Zone::Exile);
        g.move_object(a, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(insects(&g, p).len(), 4);
        assert_eq!(plus(&g, a), 0);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
#[test]
fn msh_counter_three_copied_ant_source_has_independent_draw_observer_and_paid_counter_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let a = paid(&mut g, p, ANT, &[]);
        let copy = copy_source(&mut g, p.other(), a);
        g.draw_many(p.other(), 1);
        settle(&mut g);
        assert_eq!(plus(&g, a), 0);
        assert_eq!(plus(&g, copy), 1);
        g.state.priority = p.other();
        g.state.objects[copy as usize].summoning_sick = false;
        g.state.players[p.other().index()].mana = [0, 0, 0, 0, 1, 2];
        checked(&mut g, activate(copy));
        checked(&mut g, Action::ChooseSourceCounterAmount { amount: 1 });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(plus(&g, copy), 0);
        assert_eq!(insects(&g, p.other()).len(), 1);
        assert!(insects(&g, p).is_empty());
        assert_eq!(g.state.players[p.other().index()].mana, [0; 6]);
    }
}
#[test]
fn msh_counter_three_copied_hyde_upkeep_belongs_to_its_controller_and_exact_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let h = paid(&mut g, p, HYDE, &[]);
        let copy = copy_source(&mut g, p.other(), h);
        upkeep(&mut g, p.other());
        assert_eq!(g.actor(), Some(p.other()));
        checked(&mut g, Action::ChooseMode { mode: 0 });
        settle(&mut g);
        assert_eq!(plus(&g, copy), 1);
        assert_eq!(plus(&g, h), 0);
    }
}
#[test]
fn msh_counter_three_doc_paid_counter_spell_changes_exact_one_color_power_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let d = paid(&mut g, p, DOC, &[]);
        paid(&mut g, p, "Snakeskin Veil", &[Target::Card(d)]);
        assert_eq!(plus(&g, d), 2);
        assert_eq!(g.power(d), 5);
        g.state.objects[d as usize].summoning_sick = false;
        let action = Action::ActivateMana {
            card: d,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 5, 0, 0],
        };
        g = orphan(g, action);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 5, 0, 0]);
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn msh_counter_three_doc_copy_qualifies_its_controller_and_independent_mana_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let d = paid(&mut g, p, DOC, &[]);
        let copy = copy_source(&mut g, p.other(), d);
        let r = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.effect_counter_by(
            r,
            CounterKind::PlusOne,
            1,
            p.other(),
            replacements::ResumeMode::Pending,
        );
        g.advance_replacements(false);
        settle(&mut g);
        assert_eq!(plus(&g, r), 2);
        g.state.priority = p.other();
        g.state.players[p.other().index()].mana = [0; 6];
        g.state.objects[copy as usize].summoning_sick = false;
        checked(
            &mut g,
            Action::ActivateMana {
                card: copy,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 3, 0, 0, 0, 0],
            },
        );
        assert_eq!(g.state.players[p.other().index()].mana, [0, 3, 0, 0, 0, 0]);
        assert!(!g.object_view(d).tapped);
    }
}
#[test]
fn msh_counter_three_all_three_paid_upkeep_draw_replacement_and_counter_cost_interact() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let d = paid(&mut g, p, DOC, &[]);
        let h = paid(&mut g, p, HYDE, &[]);
        let a = paid(&mut g, p, ANT, &[]);
        upkeep(&mut g, p);
        checked(&mut g, Action::ChooseMode { mode: 0 });
        settle(&mut g);
        assert_eq!(plus(&g, h), 2);
        g.state.turn += 2;
        upkeep(&mut g, p);
        checked(&mut g, Action::ChooseMode { mode: 1 });
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        let object = g.object_ref(h);
        g = orphan(
            g,
            Action::ChooseCreatureCounterCost {
                object,
                counter: CounterKind::PlusOne,
            },
        );
        settle(&mut g);
        assert_eq!(plus(&g, h), 1);
        assert_eq!(plus(&g, a), 2);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        g.state.step = Step::Main1;
        paid(&mut g, p, "Snakeskin Veil", &[Target::Card(d)]);
        assert_eq!(g.power(d), 5);
        g.state.objects[d as usize].summoning_sick = false;
        g.state.objects[a as usize].summoning_sick = false;
        checked(
            &mut g,
            Action::ActivateMana {
                card: d,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 5, 0],
            },
        );
        checked(&mut g, activate(a));
        checked(&mut g, Action::ChooseSourceCounterAmount { amount: 2 });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(plus(&g, a), 0);
        assert_eq!(insects(&g, p).len(), 2);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 2, 0]);
    }
}
