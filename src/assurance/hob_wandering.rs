//! Owner-directed source shuffle, independent draw continuation and printed ward.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("wandering decision budget: {:?}", g.state.pending);
}
fn drive(g: &mut Game) {
    until(g, |g| {
        g.outcome().is_some()
            || (g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none())
    });
}
fn source(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Gandalf, Wandering Wizard", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 4];
    cast(g, id, None);
    drive(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(stats(g, id), (4, 5));
    id
}
fn activate(g: &mut Game, card: CardId, p: Player) {
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 6];
    act(
        g,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
fn private_draws(g: &Game, p: Player, n: usize) {
    let hand: Vec<_> = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| (o.zone == Zone::Hand && o.owner == p).then_some(i as CardId))
        .collect();
    assert_eq!(hand.len(), n);
    for id in hand {
        assert!(g.observe(p).objects.iter().any(|o| o.id == id));
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == id));
    }
}
#[test]
fn wandering_paid_body_ward_three_is_opponent_only_and_underpayment_is_atomic() {
    for p in [Player::P0, Player::P1] {
        for paying in [false, true] {
            let mut g = scenario(p);
            let gandalf = source(&mut g, p);
            g.state.priority = p.other();
            let spell = put(&mut g, p.other(), "Shock", Zone::Hand);
            g.state.players[p.other().index()].mana = [0, 0, 0, 1, 0, 3];
            cast(&mut g, spell, Some(Target::Card(gandalf)));
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::ResolutionMana { .. })
            });
            reject(
                &mut g,
                Action::PayResolutionMana {
                    payment: [0, 0, 0, 0, 0, 2],
                },
            );
            act(
                &mut g,
                if paying {
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 3],
                    }
                } else {
                    Action::DeclineResolutionMana
                },
            );
            drive(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(
                g.state.objects[gandalf as usize].damage,
                if paying { 2 } else { 0 }
            );
            assert_eq!(
                g.state.players[p.other().index()].mana,
                [0, 0, 0, 0, 0, if paying { 0 } else { 3 }]
            );
            g.state.priority = p;
            let own = put(&mut g, p, "Shock", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
            cast(&mut g, own, Some(Target::Card(gandalf)));
            drive(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.state.objects[gandalf as usize].damage,
                if paying { 4 } else { 2 }
            );
        }
    }
}
#[test]
fn wandering_six_mana_instant_untapped_cost_free_and_owner_correct_after_control_change() {
    for owner in [Player::P0, Player::P1] {
        for stolen in [false, true] {
            let mut g = scenario(owner);
            let gandalf = source(&mut g, owner);
            let controller = if stolen { owner.other() } else { owner };
            if stolen {
                let object = g.object_ref(gandalf);
                g.record_control(object, controller, None);
            }
            g.state.active = controller.other();
            g.state.step = Step::End;
            g.state.priority = controller;
            g.state.objects[gandalf as usize].tapped = true;
            g.state.objects[gandalf as usize].summoning_sick = true;
            g.state.players[controller.index()].mana = [0, 0, 0, 0, 0, 5];
            reject(
                &mut g,
                Action::AnnounceActivate {
                    card: gandalf,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            let before = g.state.rng;
            activate(&mut g, gandalf, controller);
            drive(&mut g);
            assert_ne!(g.state.rng, before);
            assert_eq!(g.state.players[owner.index()].cards_drawn_this_turn, 3);
            assert_eq!(
                g.state.players[owner.other().index()].cards_drawn_this_turn,
                0
            );
            assert_eq!(g.state.players[owner.index()].library.len(), 6);
            private_draws(&g, owner, 3);
            private_draws(&g, owner.other(), 0);
            assert!(matches!(
                g.object_view(gandalf).zone,
                Zone::Library | Zone::Hand
            ));
            assert!(
                !g.state.players[owner.other().index()]
                    .library
                    .contains(&gandalf)
            );
        }
    }
}
#[test]
fn wandering_source_departed_and_reentered_never_moves_new_incarnation_but_shuffles_and_draws() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let gandalf = source(&mut g, p);
        activate(&mut g, gandalf, p);
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut valid = g.clone();
        let mut departed = g.clone();
        let mut reentered = g.clone();
        drop(g);
        drive(&mut valid);
        assert_eq!(valid.state.players[p.index()].cards_drawn_this_turn, 3);
        departed.move_object(gandalf, Zone::Graveyard);
        departed.give_priority();
        let rng = departed.state.rng;
        drive(&mut departed);
        assert_ne!(departed.state.rng, rng);
        assert_eq!(departed.object_view(gandalf).zone, Zone::Graveyard);
        assert_eq!(departed.state.players[p.index()].cards_drawn_this_turn, 3);
        assert_eq!(departed.state.players[p.index()].library.len(), 5);
        reentered.move_object(gandalf, Zone::Hand);
        reentered.move_object(gandalf, Zone::Battlefield);
        reentered.give_priority();
        let fresh = reentered.object_ref(gandalf);
        drive(&mut reentered);
        assert_eq!(reentered.object_ref(gandalf), fresh);
        assert_eq!(reentered.object_view(gandalf).zone, Zone::Battlefield);
        assert_eq!(reentered.state.players[p.index()].cards_drawn_this_turn, 3);
        reentered.restore(&saved).unwrap();
        drive(&mut reentered);
        assert_eq!(reentered.snapshot(), valid.snapshot());
    }
}
#[test]
fn wandering_repeat_stack_activations_each_shuffle_and_draw_three_without_tap_or_limit() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let gandalf = source(&mut g, p);
        activate(&mut g, gandalf, p);
        activate(&mut g, gandalf, p);
        drive(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 6);
        assert_eq!(g.state.players[p.index()].library.len(), 3);
        private_draws(&g, p, 6);
    }
}
#[test]
fn wandering_owner_draw_uses_existing_private_miracle_continuation_and_owned_restore() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let gandalf = source(&mut g, p);
        put(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
        let old = g.state.players[p.index()].library.clone();
        for id in old {
            g.move_object(id, Zone::Outside);
        }
        for _ in 0..5 {
            put(&mut g, p, "Shock", Zone::Library);
        }
        activate(&mut g, gandalf, p);
        g.move_object(gandalf, Zone::Exile);
        g.give_priority();
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::MiracleReveal { .. })
        });
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert!(g.observe(p).miracle.unwrap().can_reveal);
        assert!(g.observe(p.other()).miracle.is_none());
        contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut child = g.clone();
        drop(g);
        act(&mut child, Action::DeclineMiracle);
        drive(&mut child);
        assert_eq!(child.state.players[p.index()].cards_drawn_this_turn, 3);
        private_draws(&child, p, 3);
        let expected = child.snapshot();
        child.restore(&saved).unwrap();
        act(&mut child, Action::DeclineMiracle);
        drive(&mut child);
        assert_eq!(child.snapshot(), expected);
    }
}
#[test]
fn wandering_short_library_draw_attempts_still_lose_after_source_was_shuffled() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let gandalf = source(&mut g, p);
        let old = g.state.players[p.index()].library.clone();
        for id in old {
            g.move_object(id, Zone::Outside);
        }
        activate(&mut g, gandalf, p);
        drive(&mut g);
        assert_eq!(g.object_view(gandalf).zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert!(g.state.players[p.index()].attempted_empty_draw);
        assert_eq!(g.outcome(), Some(Outcome::Winner(p.other())));
    }
}
