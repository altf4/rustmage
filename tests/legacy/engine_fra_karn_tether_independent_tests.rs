use super::*;

fn pay_karn(g: &mut Game, karn: CardId) {
    act(g, Action::AnnounceCast { card: karn });
    for _ in 0..16 {
        let actions = g.legal_actions();
        if let Some(a) = actions
            .iter()
            .find(|a| matches!(a, Action::ChooseSpellCost { .. }))
            .cloned()
        {
            act(g, a);
        } else if actions.iter().any(|a| matches!(a, Action::PayCosts { .. })) {
            pay(g);
            return;
        } else {
            panic!("unexpected Karn cost state {:?}", g.state.pending);
        }
    }
    panic!("Karn payment did not complete");
}

#[test]
fn karn_two_brid_costs_accept_generic_all_colors_and_mixed_but_not_insufficient() {
    for p in [Player::P0, Player::P1] {
        for mana in [[0, 0, 0, 0, 0, 10], [1, 1, 1, 1, 1, 0], [1, 1, 0, 0, 0, 6]] {
            let mut g = scenario(p);
            let karn = put(&mut g, p, "Karn, Gilded Guardian", Zone::Hand);
            g.state.players[p.index()].mana = mana;
            pay_karn(&mut g, karn);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(g.object_view(karn).zone, Zone::Battlefield);
            assert_eq!(stats(&g, karn), (5, 5));
        }
        let mut g = scenario(p);
        let karn = put(&mut g, p, "Karn, Gilded Guardian", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 9];
        let saved = g.snapshot();
        reject(&mut g, Action::AnnounceCast { card: karn });
        assert_eq!(g.snapshot(), saved);
    }
}

#[test]
fn karn_counts_live_other_artifact_colors_when_entry_trigger_resolves() {
    for p in [Player::P0, Player::P1] {
        for keep in [false, true] {
            let mut g = scenario(p);
            let artifact = put(&mut g, p, "Wildfire Wickerfolk", Zone::Battlefield);
            let karn = put(&mut g, p, "Karn, Gilded Guardian", Zone::Hand);
            g.state.players[p.index()].mana = [1, 1, 1, 1, 1, 0];
            let initial = g.state.players[p.index()].library.len();
            pay_karn(&mut g, karn);
            for _ in 0..48 {
                if g.object_view(karn).zone == Zone::Battlefield {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            assert_eq!(g.object_view(karn).zone, Zone::Battlefield);
            if !keep {
                g.change_controller(g.object_ref(artifact), p.other());
            }
            g.move_object(karn, Zone::Exile);
            let saved = g.snapshot();
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle(&mut g);
                assert_eq!(
                    g.state.players[p.index()].library.len(),
                    initial - if keep { 2 } else { 0 }
                );
            }
        }
    }
}

#[test]
fn tether_empty_hand_cannot_pay_optional_discard_or_create_reflexive_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let tether = put(&mut g, p, "Tether Technician", Zone::Hand);
        let life = g.state.players[p.other().index()].life;
        cast(&mut g, tether, None);
        for _ in 0..64 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            assert!(
                !actions
                    .iter()
                    .any(|a| matches!(a, Action::ChooseTarget { .. }))
            );
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
            {
                break;
            }
            let action = actions
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass | Action::PutTrigger { .. } | Action::DeclineEffectPayment
                    )
                })
                .expect("empty-hand trigger continuation");
            act(&mut g, action);
        }
        assert_eq!(g.state.players[p.other().index()].life, life);
        assert!(g.state.stack.is_empty());
    }
}

#[test]
fn tether_discard_reflexive_damage_obeys_actual_source_prevention() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let spare = put(&mut g, p, "Forest", Zone::Hand);
        let tether = put(&mut g, p, "Tether Technician", Zone::Hand);
        let preventer = put(&mut g, p.other(), "Forest", Zone::Battlefield);
        let source = g.object_ref(preventer);
        let life = g.state.players[p.other().index()].life;
        cast(&mut g, tether, None);
        let mut paid = false;
        let mut targeted = false;
        for _ in 0..64 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            if !paid && actions.contains(&Action::Discard { card: spare }) {
                act(&mut g, Action::Discard { card: spare });
                paid = true;
            } else if paid
                && !targeted
                && actions.contains(&Action::ChooseTarget {
                    target: Target::Player(p.other()),
                })
            {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
                targeted = true;
                if g.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                }
                break;
            } else {
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
        }
        assert!(paid && targeted);
        let recipient = g.object_ref(tether);
        let timestamp = g.next_role_timestamp();
        g.state
            .roles
            .get_or_insert_default()
            .source_linked_effects
            .push(SourceLinkedEffectView {
                source,
                recipient,
                controller: p.other(),
                kind: SourceLinkedEffectKind::PreventDamageBy,
                timestamp,
            });
        for _ in 0..64 {
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
            {
                break;
            }
            choose(&mut g, |a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::ChooseDamagePrevention { .. }
                        | Action::ChooseReplacement { .. }
                )
            });
        }
        assert_eq!(g.state.players[p.other().index()].life, life);
        assert_eq!(g.object_view(spare).zone, Zone::Graveyard);
    }
}
