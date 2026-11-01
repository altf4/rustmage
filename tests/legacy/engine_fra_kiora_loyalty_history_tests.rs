use super::*;

fn walker(g: &mut Game, p: Player) -> CardId {
    g.create_tokens(p, Card::JacePlaneswalkerToken, 1, false);
    g.state
        .objects
        .iter()
        .enumerate()
        .find_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(id as CardId, "Jace"))
            .then_some(id as CardId)
        })
        .unwrap()
}

#[test]
fn loyalty_history_records_committed_player_and_resets_with_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let jace = walker(&mut g, p);
        g.state.objects[jace as usize].counters.loyalty = 5;
        let action = Action::AnnounceActivate {
            card: jace,
            ability: 1,
            counter_payment: Counters::default(),
        };
        let before = g.snapshot();
        assert!(!g.observe(p.other()).players[p.index()].loyalty_ability_activated_this_turn);
        act(&mut g, action.clone());
        pay(&mut g);
        assert!(g.observe(p.other()).players[p.index()].loyalty_ability_activated_this_turn);
        assert!(!g.observe(p).players[p.other().index()].loyalty_ability_activated_this_turn);
        super::super::contracts::branch_contract(g.clone());
        g.restore(&before).unwrap();
        assert!(!g.observe(p.other()).players[p.index()].loyalty_ability_activated_this_turn);
        act(&mut g, action);
        pay(&mut g);
        settle(&mut g);
        g.finish_cleanup();
        assert!(!g.observe(p.other()).players[p.index()].loyalty_ability_activated_this_turn);
    }
}

#[test]
fn kiora_attack_requires_a_real_loyalty_activation_and_targets_an_attacker() {
    for p in [Player::P0, Player::P1] {
        for activated in [false, true] {
            let mut g = scenario(p);
            let kiora = put(&mut g, p, "Kiora of Salt and Sand", Zone::Battlefield);
            let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let idle = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            if activated {
                let jace = walker(&mut g, p);
                g.state.objects[jace as usize].counters.loyalty = 5;
                act(
                    &mut g,
                    Action::AnnounceActivate {
                        card: jace,
                        ability: 1,
                        counter_payment: Counters::default(),
                    },
                );
                pay(&mut g);
                settle(&mut g);
            }
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            act(&mut g, Action::Attack { card: bear });
            act(&mut g, Action::FinishAttackers);
            if !activated {
                assert!(
                    !g.state
                        .pending_triggers
                        .iter()
                        .chain(g.state.stack.iter())
                        .any(|item| item.source.is_some_and(|source| source.id == kiora))
                );
                continue;
            }
            for _ in 0..48 {
                if g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(bear),
                }) {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(idle),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bear),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            g.move_object(kiora, Zone::Graveyard);
            let before = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&before).unwrap();
                }
                settle(&mut g);
                assert!(!g.object_view(bear).tapped);
                assert!(g.keywords(bear).contains(Keywords::CANT_BE_BLOCKED));
            }
        }
    }
}

#[test]
fn kiora_grants_loyalty_minus_eight_and_creates_hexproof_leviathan() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let kiora = put(&mut g, p, "Kiora of Salt and Sand", Zone::Battlefield);
        let jace = walker(&mut g, p);
        g.state.objects[jace as usize].counters.loyalty = 7;
        let action = Action::AnnouncePlaneswalkerGrant {
            source: g.object_ref(jace),
            grantor: g.object_ref(kiora),
            ability: 0,
        };
        reject(&mut g, action.clone());
        assert!(!g.observe(p.other()).players[p.index()].loyalty_ability_activated_this_turn);
        g.state.objects[jace as usize].counters.loyalty = 9;
        assert!(g.legal_actions().contains(&action));
        act(&mut g, action);
        pay(&mut g);
        assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 1);
        assert!(g.observe(p.other()).players[p.index()].loyalty_ability_activated_this_turn);
        g.move_object(kiora, Zone::Graveyard);
        settle(&mut g);
        let leviathan = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.is_token()
                    && o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.card == Card::test_index(5455))
                .then_some(id as CardId)
            })
            .unwrap();
        assert_eq!(stats(&g, leviathan), (8, 8));
        assert!(g.keywords(leviathan).contains(Keywords::HEXPROOF));
    }
}
