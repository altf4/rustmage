//! Focused printed outcomes; shared engine invariants live in core tests.
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
    {
        let p = Player::P0;
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
        {
            let mut g = scenario(owner);
            let gandalf = source(&mut g, owner);
            let controller = { owner };

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
