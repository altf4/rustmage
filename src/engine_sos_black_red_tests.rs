//! Whole-card SOS rules tests using the bounded assurance scenario helpers.
use super::*;

fn until_choice(g: &mut Game) {
    for _ in 0..12 {
        if g.state.pending != Pending::Priority {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("expected a suspended resolution choice");
}

#[test]
fn sos_dissection_independent_optional_targets_and_cleanup() {
    for p in [Player::P0, Player::P1] {
        for (plus, minus) in [(false, false), (true, false), (false, true), (true, true)] {
            let mut g = scenario(p);
            let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Dissection Practice", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: spell });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            if plus {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(a),
                    },
                );
            } else {
                act(&mut g, Action::FinishTargets);
            }
            if minus {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(b),
                    },
                );
            } else {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 21);
            assert_eq!(g.state.players[p.other().index()].life, 19);
            assert_eq!(stats(&g, a), if plus { (3, 3) } else { (2, 2) });
            assert_eq!(stats(&g, b), if minus { (1, 1) } else { (2, 2) });
            g.finish_cleanup();
            assert_eq!(stats(&g, a), (2, 2));
            assert_eq!(stats(&g, b), (2, 2));
        }
        let mut g = scenario(p);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Dissection Practice", Zone::Hand);
        announce(
            &mut g,
            spell,
            &[
                Target::Player(p.other()),
                Target::Card(bear),
                Target::Card(bear),
            ],
        );
        settle(&mut g);
        assert_eq!(
            stats(&g, bear),
            (2, 2),
            "one creature may occupy both occurrences of target"
        );
    }
}

#[test]
fn sos_dissection_partially_invalid_targets_preserve_other_effects() {
    let p = Player::P0;
    let mut g = scenario(p);
    let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let b = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let spell = put(&mut g, p, "Dissection Practice", Zone::Hand);
    let bounce = put(&mut g, p.other(), "Unsummon", Zone::Hand);
    announce(
        &mut g,
        spell,
        &[Target::Player(p.other()), Target::Card(a), Target::Card(b)],
    );
    act(&mut g, Action::Pass);
    cast(&mut g, bounce, Some(Target::Card(a)));
    settle(&mut g);
    assert_eq!(g.object_view(a).zone, Zone::Hand);
    assert_eq!(stats(&g, b), (1, 1));
    assert_eq!(g.state.players[p.index()].life, 21);
    assert_eq!(g.state.players[p.other().index()].life, 19);
}

#[test]
fn sos_flashback_grants_exact_card_full_cost_exile_and_expiry() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let shock = put(&mut g, p, "Shock", Zone::Graveyard);
        let other = put(&mut g, p.other(), "Shock", Zone::Graveyard);
        let bear = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        let spell = put(&mut g, p, "Flashback", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        for illegal in [other, bear] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(illegal),
                },
            );
        }
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(shock),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert!(g.has_graveyard_cast(p, shock, GraveyardCastKind::Flashback));
        let saved_mana: u32 = g.state.players[p.index()].mana.iter().sum();
        act(
            &mut g,
            Action::AnnounceGrantedFlashback {
                card: shock,
                flash_surcharge: false,
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        pay(&mut g);
        assert_eq!(
            saved_mana - g.state.players[p.index()].mana.iter().sum::<u32>(),
            1
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.object_view(shock).zone, Zone::Exile);
        assert!(!g.has_graveyard_cast(p, shock, GraveyardCastKind::Flashback));

        let retained = put(&mut g, p, "Shock", Zone::Graveyard);
        let grant = put(&mut g, p, "Flashback", Zone::Hand);
        cast(&mut g, grant, Some(Target::Card(retained)));
        settle(&mut g);
        assert!(g.has_graveyard_cast(p, retained, GraveyardCastKind::Flashback));
        g.finish_cleanup();
        assert!(!g.has_graveyard_cast(p, retained, GraveyardCastKind::Flashback));
    }
}

#[test]
fn sos_mica_optional_artifact_payment_copy_retarget_and_ward() {
    for p in [Player::P0, Player::P1] {
        for sacrifice in [false, true] {
            let mut g = scenario(p);
            let mica = put(&mut g, p, "Mica, Reader of Ruins", Zone::Battlefield);
            let artifact = put(&mut g, p, "Arcane Signet", Zone::Battlefield);
            let shock = put(&mut g, p, "Shock", Zone::Hand);
            cast(&mut g, shock, Some(Target::Player(p.other())));
            until_choice(&mut g);
            if sacrifice {
                let object = g.object_ref(artifact);
                act(&mut g, Action::Sacrifice { object });
                assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
                act(
                    &mut g,
                    Action::SetStackTarget {
                        index: 0,
                        target: Target::Player(p),
                    },
                );
                act(&mut g, Action::FinishStackTargets);
            } else {
                act(&mut g, Action::DeclineEffectPayment);
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, 18);
            assert_eq!(
                g.state.players[p.index()].life,
                if sacrifice { 18 } else { 20 }
            );
            assert_eq!(g.object_view(mica).zone, Zone::Battlefield);
        }
        for pay_life in [false, true] {
            let mut g = scenario(p);
            let mica = put(&mut g, p, "Mica, Reader of Ruins", Zone::Battlefield);
            let murder = put(&mut g, p.other(), "Murder", Zone::Hand);
            act(&mut g, Action::Pass);
            cast(&mut g, murder, Some(Target::Card(mica)));
            until_choice(&mut g);
            act(
                &mut g,
                if pay_life {
                    Action::PayResolutionLife
                } else {
                    Action::DeclineResolutionLife
                },
            );
            settle(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if pay_life { 17 } else { 20 }
            );
            assert_eq!(
                g.object_view(mica).zone,
                if pay_life {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}
