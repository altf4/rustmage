use super::*;

fn grant_lyra(g: &mut Game, lyra: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: lyra,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
    settle(g);
}
fn finish_lyra_damage_replacements(g: &mut Game) {
    if g.state.replacements.is_some() {
        g.advance_replacements(true);
    }
    for _ in 0..32 {
        if g.state.replacements.is_none() {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::ChooseReplacement { .. } | Action::ChooseDamagePrevention { .. }
            )
        });
    }
    panic!("damage replacements did not finish");
}

#[test]
fn lyra_unblocked_double_strike_combat_triggers_once_per_damage_step() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let lyra = put(&mut g, p, "Lyra, Tolarian Archangel", Zone::Battlefield);
        grant_lyra(&mut g, lyra);
        g.state.objects[lyra as usize].gained_keywords = Keywords::DOUBLE_STRIKE;
        let hand = g.hand_count(p);
        let life = g.state.players[p.other().index()].life;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: lyra });
        act(&mut g, Action::FinishAttackers);
        for _ in 0..32 {
            if g.state.pending == Pending::Blockers {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert_eq!(g.state.pending, Pending::Blockers);
        act(&mut g, Action::FinishBlockers);
        for _ in 0..80 {
            if g.hand_count(p) == hand + 4 {
                break;
            }
            let actions = g.legal_actions();
            let next = actions
                .iter()
                .find(|a| matches!(a, Action::PutTrigger { .. }))
                .or_else(|| actions.iter().find(|a| matches!(a, Action::Pass)))
                .cloned()
                .unwrap_or_else(|| panic!("unexpected combat state {:?}", g.state.pending));
            act(&mut g, next);
        }
        assert_eq!(g.hand_count(p), hand + 4);
        assert_eq!(g.state.players[p.other().index()].life, life - 6);
    }
}

#[test]
fn lyra_prevented_combat_damage_does_not_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let lyra = put(&mut g, p, "Lyra, Tolarian Archangel", Zone::Battlefield);
        grant_lyra(&mut g, lyra);
        let prevention = put(&mut g, p.other(), "Forest", Zone::Battlefield);
        let source = g.object_ref(prevention);
        let recipient = g.object_ref(lyra);
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
        let life = g.state.players[p.other().index()].life;
        let event = DamageEvent {
            source: g.damage_source(lyra),
            target: Target::Player(p.other()),
            amount: 3,
            combat: true,
            preventable: true,
        };
        g.apply_damage_batch(&[event]);
        finish_lyra_damage_replacements(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, life);
        assert!(g.state.pending_triggers.is_empty());
    }
}

#[test]
fn lyra_queued_trigger_keeps_damage_time_controller_after_departure_or_control_change() {
    for p in [Player::P0, Player::P1] {
        for leave in [false, true] {
            let mut g = scenario(p);
            let lyra = put(&mut g, p, "Lyra, Tolarian Archangel", Zone::Battlefield);
            grant_lyra(&mut g, lyra);
            let hand = g.hand_count(p);
            let other_hand = g.hand_count(p.other());
            let event = DamageEvent {
                source: g.damage_source(lyra),
                target: Target::Player(p.other()),
                amount: 1,
                combat: true,
                preventable: true,
            };
            g.apply_damage_batch(&[event]);
            assert_eq!(g.state.pending_triggers.len(), 1);
            if leave {
                g.move_object(lyra, Zone::Exile);
            } else {
                g.change_controller(g.object_ref(lyra), p.other());
            }
            let snapshot = g.snapshot();
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&snapshot).unwrap();
                }
                g.prepare_triggers();
                settle(&mut g);
                assert_eq!(g.hand_count(p), hand + 2);
                assert_eq!(g.hand_count(p.other()), other_hand);
            }
        }
    }
}
