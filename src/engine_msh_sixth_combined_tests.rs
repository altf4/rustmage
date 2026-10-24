//! The six whole candidates interact through actual paid costs and owned replay.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
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
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .unwrap_or_else(|| panic!("combined stalled {:?}", g.state.pending));
        checked(g, action);
    }
    panic!("combined stalled {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    for _ in 0..16 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseSpellCost { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                )
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unpaid combined announcement");
}
fn loot(g: &mut Game, p: Player) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectDiscard { .. })
    });
    assert_eq!(g.actor(), Some(p));
    let card = g
        .legal_actions()
        .into_iter()
        .find_map(|a| match a {
            Action::Discard { card } if g.state.objects[card as usize].card == Card::Forest => {
                Some(card)
            }
            _ => None,
        })
        .unwrap();
    checked(g, Action::Discard { card });
}
#[test]
fn msh_sixth_combined_discard_enables_guard_tap_loot_recovers_punch_before_received_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        // The opposing back face is paid during its controller's setup turn.
        g.state.active = p.other();
        g.state.priority = p.other();
        let she = named(&mut g, p.other(), "Jennifer Walters", Zone::Hand);
        checked(&mut g, Action::AnnounceBack { card: she });
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(she).card, Card::MshSensationalSheHulk);
        g.state.active = p;
        g.state.priority = p;
        let plan = named(&mut g, p, "Rewrite History", Zone::Hand);
        cast(&mut g, plan, None, &[]);
        settle(&mut g);
        let arnim = named(&mut g, p, "Arnim Zola, Bio-Fanatic", Zone::Hand);
        cast(&mut g, arnim, None, &[]);
        settle(&mut g);
        let bull = named(&mut g, p, "Bullseye, Death Dealer", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: bull });
        finish(&mut g);
        until(&mut g, |g| {
            g.legal_actions().contains(&Action::DeclineEffectPayment)
        });
        checked(&mut g, Action::DeclineEffectPayment);
        settle(&mut g);
        for id in [arnim, bull] {
            g.state.objects[id as usize].summoning_sick = false;
        }
        let first = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let second = named(&mut g, p, "Bear Cub", Zone::Hand);
        let kun = named(&mut g, p, "K'un-Lun Warrior", Zone::Hand);
        cast(&mut g, kun, None, &[]);
        until(&mut g, |g| {
            g.legal_actions()
                .contains(&Action::Discard { card: second })
        });
        checked(&mut g, Action::Discard { card: second });
        settle(&mut g);
        assert_eq!(g.object_view(first).zone, Zone::Graveyard);
        assert_eq!(g.object_view(second).zone, Zone::Graveyard);
        g.set_counters(plan, Counters::of(CounterKind::Plan, 2));
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: arnim,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        finish(&mut g);
        loot(&mut g, p);
        settle(&mut g);
        assert_eq!(g.counter_count(plan, CounterKind::Plan), 3);
        let villains: Vec<_> = g
            .state
            .objects
            .iter()
            .filter(|o| {
                o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.is_token()
                    && o.card == Card(5312)
            })
            .collect();
        assert_eq!(villains.len(), 1);
        assert!(villains[0].tapped);
        // The token entered tapped; it did not create another tap event.
        let victim = named(&mut g, p.other(), "Aegis Turtle", Zone::Battlefield);
        let punch = named(&mut g, p, "Punishing Punch", Zone::Hand);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: bull,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let payment_object = g.object_ref(punch);
        checked(
            &mut g,
            Action::ChooseAdditionalCost {
                object: payment_object,
            },
        );
        finish(&mut g);
        assert_eq!(g.object_view(punch).zone, Zone::Graveyard);
        assert!(g.object_view(bull).tapped);
        loot(&mut g, p);
        until(&mut g, |g| {
            g.state.announcing.as_ref().is_some_and(|a| {
                a.item.card == plan && a.item.kind == (StackKind::TriggeredAbility { ability: 2 })
            })
        });
        assert_eq!(g.object_view(plan).zone, Zone::Graveyard);
        assert_eq!(g.object_view(victim).damage, 0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(punch),
            },
        );
        finish(&mut g);
        until(&mut g, |g| {
            g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.damage_received.is_some())
        });
        assert_eq!(g.object_view(punch).zone, Zone::Hand);
        assert_eq!(g.object_view(victim).damage, 2);
        let received = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .item
            .damage_received
            .as_ref()
            .unwrap();
        assert_eq!(
            (received.recipient, received.amount),
            (g.object_ref(victim), 2)
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bull),
            },
        );
        finish(&mut g);
        until(&mut g, |g| {
            g.legal_actions()
                .contains(&Action::ChooseOptional { accept: true })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.object_view(bull).damage, 2);
        assert_eq!(
            g.state
                .triggered_this_turn
                .iter()
                .filter(|r| r.source == g.object_ref(she))
                .count(),
            1
        );
        // Recovery happened before the suspended activated damage. The spell is
        // now cast with its real two-creature-card discount and reads live power.
        if g.actor() != Some(p) {
            checked(&mut g, Action::Pass);
        }
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        cast(
            &mut g,
            punch,
            None,
            &[Target::Card(bull), Target::Card(she)],
        );
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        until(&mut g, |g| g.object_view(she).damage == 4);
        for _ in 0..40 {
            if idle(&g) {
                break;
            }
            assert!(
                !g.legal_actions()
                    .iter()
                    .any(|a| matches!(a, Action::ChooseOptional { .. })),
                "spent per-turn allowance resurfaced"
            );
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass
                            | Action::PutTrigger { .. }
                            | Action::FinishTargets
                            | Action::PayCosts { .. }
                    )
                })
                .or_else(|| {
                    g.legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::ChooseTarget { .. }))
                })
                .unwrap();
            checked(&mut g, a);
        }
        assert!(idle(&g));
        assert_eq!(g.object_view(she).damage, 4);
        assert_eq!(g.object_view(bull).damage, 2);
        assert_eq!(g.object_view(plan).zone, Zone::Graveyard);
        assert_eq!(g.object_view(punch).zone, Zone::Graveyard);
    }
}
