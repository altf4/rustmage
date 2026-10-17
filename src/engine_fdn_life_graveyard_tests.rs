use super::tests::{act, pass_pair, scenario, spawn};
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for player in &mut g.state.players {
        player.mana = [20; 6];
    }
    g
}
fn replay(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g
        .legal_actions()
        .iter()
        .position(|a| a == &action)
        .expect("legal action");
    let before = g.snapshot();
    let mut child = {
        let parent = g.clone();
        parent.clone()
    };
    child.apply_index(actor, revision, index).unwrap();
    act(g, action.clone());
    let after = g.snapshot();
    assert_eq!(child.snapshot(), after);
    for viewer in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(child.observe(viewer)).unwrap(),
            serde_json::to_value(g.observe(viewer)).unwrap()
        );
    }
    g.restore(&before).unwrap();
    assert_eq!(
        g.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    act(g, action);
    assert_eq!(g.snapshot(), after);
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn pay(g: &mut Game) {
    for _ in 0..10 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .expect("payment/finish choice");
        replay(g, action);
    }
    panic!("announcement stalled");
}
fn announce(g: &mut Game, spell: CardId, targets: &[CardId]) {
    replay(g, Action::AnnounceCast { card: spell });
    for &id in targets {
        replay(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    pay(g);
}

#[test]
fn fdn_life_exsanguinate_paid_x_zero_and_positive_is_loss_not_damage() {
    for p in [Player::P0, Player::P1] {
        for x in [0, 1, 7, 30] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [0, 0, 2, 0, 0, x];
            // Earlier loss must not be included in this spell's gain.
            g.lose_life(p.other(), 4);
            let spell = named(&mut g, p, "Exsanguinate", Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: spell });
            reject(&mut g, Action::ChooseX { value: x + 1 });
            let before = g.snapshot();
            assert_eq!(
                g.apply(p.other(), Action::ChooseX { value: x }),
                Err(Error::WrongActor)
            );
            assert_eq!(g.snapshot(), before);
            replay(&mut g, Action::ChooseX { value: x });
            pay(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            pass_pair(&mut g);
            assert_eq!(g.state.players[p.index()].life, 20 + i128::from(x));
            assert_eq!(g.state.players[p.other().index()].life, 16 - i128::from(x));
            assert_eq!(
                g.state.players[p.index()].life_gained_this_turn,
                u128::from(x)
            );
            assert_eq!(
                g.state.players[p.other().index()].life_lost_this_turn,
                4 + u128::from(x)
            );
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(
                g.outcome(),
                if x >= 16 {
                    Some(Outcome::Winner(p))
                } else {
                    None
                }
            );
        }
    }
}

#[test]
fn fdn_life_indulgence_gross_gain_threshold_colored_cost_and_zero_targets() {
    for p in [Player::P0, Player::P1] {
        for gained in [0, 2, 3, 6] {
            let mut g = setup(p);
            // Lose more than the gain: the condition tracks gross gain, not net life.
            g.gain_life(p, gained);
            g.lose_life(p, 8);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
            let spell = named(&mut g, p, "Sanguine Indulgence", Zone::Hand);
            announce(&mut g, spell, &[]);
            assert_eq!(g.state.players[p.index()].mana[2], 0);
            assert_eq!(
                g.state.players[p.index()].mana[5],
                if gained >= 3 { 3 } else { 0 }
            );
            pass_pair(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(g.hand_count(p), 0);
        }
        let mut g = setup(p);
        g.gain_life(p, 3);
        g.state.players[p.index()].mana = [0; 6];
        let spell = named(&mut g, p, "Sanguine Indulgence", Zone::Hand);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceCast{card} if *card==spell))
        );
    }
}

#[test]
fn fdn_life_indulgence_choose_one_or_two_reject_duplicates_and_partial_revalidation() {
    for p in [Player::P0, Player::P1] {
        for count in [1, 2] {
            let mut g = setup(p);
            let first = named(&mut g, p, "Diregraf Ghoul", Zone::Graveyard);
            let second = named(&mut g, p, "Bear Cub", Zone::Graveyard);
            let foe = named(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
            let land = named(&mut g, p, "Forest", Zone::Graveyard);
            let spell = named(&mut g, p, "Sanguine Indulgence", Zone::Hand);
            replay(&mut g, Action::AnnounceCast { card: spell });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foe),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(land),
                },
            );
            replay(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(first),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(first),
                },
            );
            if count == 2 {
                replay(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(second),
                    },
                );
            }
            pay(&mut g);
            if count == 2 {
                g.move_object(first, Zone::Exile);
            }
            pass_pair(&mut g);
            assert_eq!(
                g.object_view(first).zone,
                if count == 1 { Zone::Hand } else { Zone::Exile }
            );
            assert_eq!(
                g.object_view(second).zone,
                if count == 1 {
                    Zone::Graveyard
                } else {
                    Zone::Hand
                }
            );
            let hand = if count == 1 { first } else { second };
            assert!(
                g.observe(p.other())
                    .known_hand
                    .iter()
                    .any(|entry| entry.card == hand)
            );
        }
    }
}

#[test]
fn fdn_life_indulgence_tracks_multiple_events_and_resets_at_new_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.gain_life(p, 1);
        g.gain_life(p, 2);
        assert_eq!(g.state.players[p.index()].life_gained_this_turn, 3);
        g.state.active = p.other();
        g.begin_turn();
        assert_eq!(g.state.players[p.index()].life_gained_this_turn, 0);
        // A new main-phase fixture with only B cannot pay the undiscounted spell.
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        g.state.pending = Pending::Priority;
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
        let spell = named(&mut g, p, "Sanguine Indulgence", Zone::Hand);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a,Action::AnnounceCast{card} if *card==spell))
        );
    }
}

#[test]
fn fdn_life_cemetery_returns_creature_and_draws_only_for_zombie_with_private_draw() {
    for p in [Player::P0, Player::P1] {
        for (name, draws) in [("Diregraf Ghoul", 1), ("Bear Cub", 0)] {
            let mut g = setup(p);
            let target = named(&mut g, p, name, Zone::Graveyard);
            let old = g.object_ref(target);
            let hidden = named(&mut g, p, "Giant Growth", Zone::Library);
            let spell = named(&mut g, p, "Cemetery Recruitment", Zone::Hand);
            let library = g.state.players[p.index()].library.len();
            let mana: u32 = g.state.players[p.index()].mana.iter().sum();
            announce(&mut g, spell, &[target]);
            assert_eq!(
                g.state.players[p.index()].mana.iter().sum::<u32>(),
                mana - 2
            );
            pass_pair(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Hand);
            assert_ne!(g.object_ref(target), old);
            assert_eq!(g.hand_count(p), 1 + draws);
            assert_eq!(g.state.players[p.index()].library.len(), library - draws);
            let view = g.observe(p.other());
            assert!(view.known_hand.iter().any(|entry| entry.card == target));
            assert!(!view.objects.iter().any(|o| o.id == hidden));
            assert!(!view.known_hand.iter().any(|entry| entry.card == hidden));
        }
    }
}

#[test]
fn fdn_life_cemetery_rejects_wrong_targets_and_stale_incarnation_cannot_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let target = named(&mut g, p, "Diregraf Ghoul", Zone::Graveyard);
        let foe = named(&mut g, p.other(), "Diregraf Ghoul", Zone::Graveyard);
        let land = named(&mut g, p, "Forest", Zone::Graveyard);
        let live = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Cemetery Recruitment", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: spell });
        for bad in [foe, land, live] {
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bad),
                },
            );
        }
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        let library = g.state.players[p.index()].library.len();
        g.move_object(target, Zone::Exile);
        g.move_object(target, Zone::Graveyard);
        pass_pair(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(g.hand_count(p), 0);
        assert_eq!(g.state.players[p.index()].library.len(), library);
    }
}
