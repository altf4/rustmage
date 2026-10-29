use super::*;

#[test]
fn tetsuko_forbids_actual_block_at_low_stat_threshold() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Tetsuko Umezawa, Fugitive", Zone::Battlefield);
        let small = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let normal = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let blocker = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[small as usize].growth.power = -1;
        assert!(!g.can_block_pair(blocker, small));
        g.state.objects[small as usize].growth.power = -2;
        assert!(!g.can_block_pair(blocker, small));
        g.state.objects[small as usize].growth.power = -1;
        assert!(g.can_block_pair(blocker, normal));
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: small });
        act(&mut g, Action::Attack { card: normal });
        act(&mut g, Action::FinishAttackers);
        for _ in 0..8 {
            if g.state.pending == Pending::Blockers {
                break;
            }
            act(&mut g, Action::Pass);
        }
        reject(
            &mut g,
            Action::Block {
                blocker,
                attacker: small,
            },
        );
        act(
            &mut g,
            Action::Block {
                blocker,
                attacker: normal,
            },
        );
        act(&mut g, Action::FinishBlockers);
    }
}

#[test]
fn twinned_vision_countered_flashback_still_exiles_and_paid_discard_can_be_replaced() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        let spell = put(&mut g, p, "Twinned Vision", Zone::Graveyard);
        let discard = put(&mut g, p, "Bear Cub", Zone::Hand);
        let negate = put(&mut g, p.other(), "Negate", Zone::Hand);
        let before = g.state.players[p.index()].library.len();
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 1];
        act(&mut g, Action::AnnounceFlashback { card: spell });
        for _ in 0..24 {
            let actions = g.legal_actions();
            if let Some(action) = actions
                .iter()
                .find(|a| matches!(a, Action::ChooseSpellCost { .. }))
                .cloned()
            {
                act(&mut g, action);
            } else if actions
                .iter()
                .any(|a| matches!(a, Action::ChooseAdditionalCost { .. }))
            {
                let object = g.object_ref(discard);
                act(&mut g, Action::ChooseAdditionalCost { object });
            } else if let Some(action) = actions
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
            {
                act(&mut g, action);
            } else if g.state.pending == Pending::Priority && !g.state.stack.is_empty() {
                break;
            } else {
                panic!("unexpected Twinned Vision choice {:?}", g.state.pending);
            }
        }
        assert_eq!(g.object_view(discard).zone, Zone::Exile);
        act(&mut g, Action::Pass);
        cast(&mut g, negate, Some(Target::Spell(spell)));
        super::super::contracts::branch_contract(g.clone());
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].library.len(), before);
    }
}

#[test]
fn infinite_coursework_target_fizzle_and_departed_aura_entry_trigger() {
    for p in [Player::P0, Player::P1] {
        for fizzle in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
            let aura = put(&mut g, p, "Infinite Coursework", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: aura });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            pay(&mut g);
            if fizzle {
                g.move_object(target, Zone::Exile);
                settle(&mut g);
                assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
            } else {
                for _ in 0..32 {
                    if g.object_view(aura).zone == Zone::Battlefield {
                        break;
                    }
                    choose(&mut g, |a| {
                        matches!(a, Action::Pass | Action::PutTrigger { .. })
                    });
                }
                assert_eq!(g.object_view(aura).zone, Zone::Battlefield);
                g.move_object(aura, Zone::Graveyard);
                super::super::contracts::branch_contract(g.clone());
                settle(&mut g);
                assert!(g.object_view(target).tapped);
            }
        }
    }
}

#[test]
fn jiang_doubled_mowu_tokens_obey_legend_rule() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let jiang = put(&mut g, p, "Jiang Yanggu, Never Alone", Zone::Hand);
        cast(&mut g, jiang, None);
        let mut legend = false;
        for _ in 0..64 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            if let Some(action) = actions
                .iter()
                .find(|a| matches!(a, Action::KeepLegend { .. }))
                .cloned()
            {
                assert_eq!(
                    actions
                        .iter()
                        .filter(|a| matches!(a, Action::KeepLegend { .. }))
                        .count(),
                    2
                );
                super::super::contracts::branch_contract(g.clone());
                act(&mut g, action);
                legend = true;
            } else if legend
                && g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
            {
                break;
            } else if let Some(action) = actions
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
            {
                act(&mut g, action);
            } else {
                panic!("unexpected Jiang legend choice {:?}", g.state.pending);
            }
        }
        assert!(legend);
        assert_eq!(
            g.state
                .objects
                .iter()
                .enumerate()
                .filter(|(i, o)| o.zone == Zone::Battlefield
                    && o.is_token()
                    && o.controller == p
                    && g.object_name(*i as CardId) == "Mowu")
                .count(),
            1
        );
    }
}
