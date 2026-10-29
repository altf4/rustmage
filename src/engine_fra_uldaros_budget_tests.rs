use super::*;

// A real resolving Chandra ability supplies the ordinary cast continuation;
// replace its permission with the budgeted free offer under test.
fn budget_offer(g: &mut Game, player: Player, names: &[&str], budget: u64) -> Vec<ObjectRef> {
    let chandra = put(g, player, "Chandra, Torch of Defiance", Zone::Battlefield);
    g.state.objects[chandra as usize].counters.loyalty = 4;
    let cards: Vec<_> = names
        .iter()
        .map(|name| put(g, player, name, Zone::Exile))
        .collect();
    put(g, player, "Mountain", Zone::Library);
    act(
        g,
        Action::AnnounceActivate {
            card: chandra,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
    g.resolve_top();
    let refs: Vec<_> = cards.iter().map(|&id| g.object_ref(id)).collect();
    let offer = g.state.resolution_free_casts.as_mut().unwrap();
    offer.paid = None;
    offer.casts_remaining = None;
    offer.remaining_mana_value = Some(budget);
    offer.remaining = refs.clone();
    refs
}

#[test]
fn budget_is_inclusive_cumulative_and_replays_for_both_seats() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let cards = budget_offer(&mut g, player, &["Bear Cub", "Hill Giant", "Bear Cub"], 6);
        let saved = g.snapshot();
        for _ in 0..2 {
            let a = Action::CastExiledDuringResolution {
                object: cards[0],
                face: CastFace::Normal,
            };
            assert!(g.legal_actions().contains(&a));
            act(&mut g, a);
            pay(&mut g);
            assert_eq!(
                g.state
                    .resolution_free_casts
                    .as_ref()
                    .unwrap()
                    .remaining_mana_value,
                Some(4)
            );
            let b = Action::CastExiledDuringResolution {
                object: cards[1],
                face: CastFace::Normal,
            };
            assert!(g.legal_actions().contains(&b));
            act(&mut g, b);
            pay(&mut g);
            assert_eq!(
                g.state
                    .resolution_free_casts
                    .as_ref()
                    .unwrap()
                    .remaining_mana_value,
                Some(0)
            );
            assert!(
                !g.legal_actions()
                    .contains(&Action::CastExiledDuringResolution {
                        object: cards[2],
                        face: CastFace::Normal
                    })
            );
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn budget_uses_selected_adventure_face_and_rejects_over_budget() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let cards = budget_offer(
            &mut g,
            player,
            &["Great Ugly-Looking Goblin // Clap! Snap!"],
            2,
        );
        let object = cards[0];
        assert!(
            !g.legal_actions()
                .contains(&Action::CastExiledDuringResolution {
                    object,
                    face: CastFace::Normal
                })
        );
        let saved = g.snapshot();
        for _ in 0..2 {
            let a = Action::CastExiledDuringResolution {
                object,
                face: CastFace::Adventure,
            };
            assert!(g.legal_actions().contains(&a));
            act(&mut g, a);
            pay(&mut g);
            assert_eq!(g.state.stack.last().unwrap().mana_value(), 2);
            assert_eq!(
                g.state
                    .resolution_free_casts
                    .as_ref()
                    .unwrap()
                    .remaining_mana_value,
                Some(0)
            );
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn budget_free_x_is_zero_and_invalid_actions_leave_state_unchanged() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let cards = budget_offer(&mut g, player, &["Fblthp, Knows the Way", "Hill Giant"], 2);
        let impossible = Action::CastExiledDuringResolution {
            object: cards[1],
            face: CastFace::Normal,
        };
        let saved = g.snapshot();
        reject(&mut g, impossible);
        assert_eq!(g.snapshot(), saved);
        for _ in 0..2 {
            let action = Action::CastExiledDuringResolution {
                object: cards[0],
                face: CastFace::Normal,
            };
            assert!(g.legal_actions().contains(&action));
            act(&mut g, action);
            pay(&mut g);
            let spell = g.state.stack.last().unwrap();
            assert_eq!(spell.x, Some(0));
            assert_eq!(spell.mana_value(), 2);
            assert_eq!(
                g.state
                    .resolution_free_casts
                    .as_ref()
                    .unwrap()
                    .remaining_mana_value,
                Some(0)
            );
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn budget_is_public_for_exile_offer_and_retained_by_snapshots() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        budget_offer(&mut g, player, &["Bear Cub"], 6);
        let saved = g.snapshot();
        for viewer in [Player::P0, Player::P1] {
            let json = serde_json::to_value(g.observe(viewer)).unwrap();
            assert_eq!(json["resolution_free_casts"]["remaining_mana_value"], 6);
        }
        g.state
            .resolution_free_casts
            .as_mut()
            .unwrap()
            .remaining_mana_value = None;
        let json = serde_json::to_value(g.observe(player)).unwrap();
        assert!(
            json["resolution_free_casts"]
                .get("remaining_mana_value")
                .is_none()
        );
        g.restore(&saved).unwrap();
        assert_eq!(
            g.state
                .resolution_free_casts
                .as_ref()
                .unwrap()
                .remaining_mana_value,
            Some(6)
        );
    }
}

#[test]
fn budget_pending_announcement_restores_on_failed_transaction() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let cards = budget_offer(&mut g, player, &["Bear Cub"], 6);
        let before = g.snapshot();
        // Exercise the same owned checkpoint used by failed announcements;
        // this synthetic checkpoint isolates the continuation from mana replay.
        g.begin_attack_rollback(player);
        g.cast_exiled_during_resolution(player, cards[0], CastFace::Normal);
        let offer = g.state.resolution_free_casts.as_ref().unwrap();
        assert_eq!(offer.remaining_mana_value, Some(6));
        assert!(offer.pending_cast.is_some());
        g.finish_cast_rollback(false);
        assert_eq!(g.snapshot(), before);
        assert!(
            g.legal_actions()
                .contains(&Action::CastExiledDuringResolution {
                    object: cards[0],
                    face: CastFace::Normal,
                })
        );
    }
}
