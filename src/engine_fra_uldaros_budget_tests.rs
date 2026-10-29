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

#[test]
fn budget_cast_card_copy_consumes_once_on_actual_commit() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        budget_offer(&mut g, player, &[], 6);
        let bear = Card::from_name("Bear Cub").unwrap();
        let copy = g.allocate_card_copy(bear, player);
        g.state
            .resolution_free_casts
            .as_mut()
            .unwrap()
            .remaining
            .push(copy);
        let saved = g.snapshot();
        for _ in 0..2 {
            act(
                &mut g,
                Action::CastExiledDuringResolution {
                    object: copy,
                    face: CastFace::Normal,
                },
            );
            pay(&mut g);
            assert_eq!(
                g.state
                    .resolution_free_casts
                    .as_ref()
                    .unwrap()
                    .remaining_mana_value,
                Some(4)
            );
            assert!(!g.state.stack.last().unwrap().is_copy);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn budget_real_interrupted_payment_rolls_back_after_foreign_entry_choice() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let silver = put(
            &mut g,
            player,
            "Silverquill, the Disputant",
            Zone::Battlefield,
        );
        g.state.objects[silver as usize].growth.power = -1;
        let enemy = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let aura = put(
            &mut g,
            player.other(),
            "Witness Protection",
            Zone::Battlefield,
        );
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(enemy));
        let prison = put(&mut g, player, "Banishing Light", Zone::Hand);
        super::super::big_helpers::cast(&mut g, prison, None, &[]);
        for _ in 0..40 {
            if g.state.pending != Pending::Priority {
                break;
            }
            act(&mut g, Action::Pass);
        }
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(aura),
            },
        );
        super::super::big_helpers::settle(&mut g);
        assert_eq!(g.object_view(aura).zone, Zone::Exile);
        let refs = budget_offer(&mut g, player, &["Candy Grapple"], 6);
        let before = g.snapshot();
        act(
            &mut g,
            Action::CastExiledDuringResolution {
                object: refs[0],
                face: CastFace::Normal,
            },
        );
        assert_eq!(
            g.state
                .resolution_free_casts
                .as_ref()
                .unwrap()
                .remaining_mana_value,
            Some(6)
        );
        let sacrifice = g.object_ref(prison);
        act(
            &mut g,
            Action::ChooseBargain {
                sacrifice: Some(sacrifice),
            },
        );
        act(&mut g, Action::ChooseCasualty { pay: true });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        pay(&mut g);
        let clause = g
            .observe(player)
            .cast_costs
            .unwrap()
            .clauses
            .iter()
            .position(|row| row.clause == CastCostClause::Additional && !row.paid)
            .unwrap();
        act(
            &mut g,
            Action::ChooseCastCost {
                clause: clause as u16,
            },
        );
        assert_eq!(g.actor(), Some(player.other()));
        let attachment = g.object_ref(silver);
        act(&mut g, Action::ChooseAuraAttachment { object: attachment });
        assert_eq!(g.snapshot(), before);
        assert_eq!(
            g.state
                .resolution_free_casts
                .as_ref()
                .unwrap()
                .remaining_mana_value,
            Some(6)
        );
        assert!(
            g.legal_actions()
                .contains(&Action::CastExiledDuringResolution {
                    object: refs[0],
                    face: CastFace::Normal
                })
        );
    }
}
