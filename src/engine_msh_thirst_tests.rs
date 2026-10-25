//! The whole paid spell: private alternative payment and owned continuation.
use super::assurance::big_helpers::{cast, checked, named, settle, setup};
use super::*;
fn reject(g: &mut Game, a: Action) {
    let old = g.snapshot();
    let bytes = g.state_bytes();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), old);
    assert_eq!(g.state_bytes(), bytes);
}
fn until_choice(g: &mut Game) {
    for _ in 0..12 {
        if matches!(g.state.pending, Pending::EffectChoice { .. }) {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("Thirst did not offer its optional payment");
}
fn thirst(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Thirst for Knowledge", Zone::Hand);
    cast(g, id, None, &[]);
    until_choice(g);
    assert_eq!(g.actor(), Some(p));
    let choice = g.observe(p).effect_choice.unwrap();
    assert_eq!(choice.kind, EffectChoiceKind::Discard);
    assert!(choice.optional);
    assert_eq!(choice.amount, 1);
    assert_eq!(choice.required[p.index()], 1);
    assert!(g.observe(p.other()).effect_choice.is_none());
    assert!(g.outcome().is_none());
    id
}
fn remaining_library(g: &mut Game, p: Player, n: usize) {
    let remove: Vec<_> = g.state.players[p.index()]
        .library
        .iter()
        .skip(n)
        .copied()
        .collect();
    for id in remove {
        g.move_object(id, Zone::Exile);
    }
}
#[test]
fn msh_thirst_paid_instant_on_either_turn_draws_three_before_artifact_payment() {
    for p in [Player::P0, Player::P1] {
        for other_turn in [false, true] {
            let mut g = setup(p);
            g.state.active = if other_turn { p.other() } else { p };
            let artifact = named(&mut g, p, "Gilded Lotus", Zone::Hand);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
            let tops: Vec<_> = g.state.players[p.index()]
                .library
                .iter()
                .rev()
                .take(3)
                .copied()
                .collect();
            let spell = thirst(&mut g, p);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
            assert_eq!(g.object_view(spell).zone, Zone::Stack);
            for id in tops {
                assert_eq!(g.object_view(id).zone, Zone::Hand);
                assert!(g.observe(p.other()).objects.iter().all(|o| o.id != id));
            }
            checked(&mut g, Action::Discard { card: artifact });
            settle(&mut g);
            assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
            assert_eq!(g.hand_count(p), 3);
        }
    }
}
#[test]
fn msh_thirst_artifact_land_is_eligible_and_other_zones_or_nonartifacts_are_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        // Metadata-only hand fixture: this does not admit Scene's unrelated behavior.
        let land = named(&mut g, p, "Forest", Zone::Hand);
        g.state.objects[land as usize].card = Card::from_name("Scene of the Crime").unwrap();
        assert!(g.state.objects[land as usize].card.is_land());
        assert!(g.state.objects[land as usize].card.is_artifact());
        let wrong = named(&mut g, p, "Shock", Zone::Hand);
        let enemy = named(&mut g, p.other(), "Gilded Lotus", Zone::Hand);
        let battlefield = named(&mut g, p, "Gilded Lotus", Zone::Battlefield);
        let graveyard = named(&mut g, p, "Gilded Lotus", Zone::Graveyard);
        thirst(&mut g, p);
        for card in [wrong, enemy, battlefield, graveyard] {
            reject(&mut g, Action::Discard { card });
        }
        reject(&mut g, Action::Pass);
        checked(&mut g, Action::Discard { card: land });
        settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Graveyard);
        assert_eq!(g.object_view(wrong).zone, Zone::Hand);
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
    }
}
#[test]
fn msh_thirst_decline_allows_any_two_including_two_artifacts_as_one_private_batch() {
    for p in [Player::P0, Player::P1] {
        for artifacts in 0..=2 {
            let mut g = setup(p);
            let a = named(
                &mut g,
                p,
                if artifacts > 0 {
                    "Gilded Lotus"
                } else {
                    "Forest"
                },
                Zone::Hand,
            );
            let b = named(
                &mut g,
                p,
                if artifacts > 1 {
                    "Gilded Lotus"
                } else {
                    "Shock"
                },
                Zone::Hand,
            );
            thirst(&mut g, p);
            checked(&mut g, Action::DeclineEffectPayment);
            assert!(matches!(
                g.state.pending,
                Pending::EffectDiscard { remaining: 2, .. }
            ));
            checked(&mut g, Action::Discard { card: a });
            assert_eq!(g.object_view(a).zone, Zone::Hand);
            assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 0);
            // The changing remaining quota is public, but the selected identity is not.
            let after = g.observe(p.other());
            assert!(after.objects.iter().all(|o| o.id != a && o.id != b));
            reject(&mut g, Action::Discard { card: a });
            reject(&mut g, Action::DeclineEffectPayment);
            checked(&mut g, Action::Discard { card: b });
            settle(&mut g);
            assert_eq!(g.object_view(a).zone, Zone::Graveyard);
            assert_eq!(g.object_view(b).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 2);
            assert_eq!(g.hand_count(p), 3);
        }
    }
}
#[test]
fn msh_thirst_optional_phase_is_identical_publicly_with_or_without_matching_hand_card() {
    for p in [Player::P0, Player::P1] {
        let mut no = setup(p);
        let hidden = named(&mut no, p, "Forest", Zone::Hand);
        let id = named(&mut no, p, "Thirst for Knowledge", Zone::Hand);
        let mut yes = no.clone();
        yes.state.objects[hidden as usize].card = Card::from_name("Gilded Lotus").unwrap();
        for g in [&mut no, &mut yes] {
            cast(g, id, None, &[]);
            until_choice(g);
            assert_eq!(g.actor(), Some(p));
            let view = g.observe(p).effect_choice.unwrap();
            assert_eq!(view.required[p.index()], 1);
            assert!(view.optional);
        }
        assert!(no.observe(p).effect_choice.unwrap().candidates.is_empty());
        assert_eq!(
            yes.observe(p).effect_choice.unwrap().candidates,
            vec![yes.object_ref(hidden)]
        );
        assert_eq!(
            serde_json::to_value(no.observe(p.other())).unwrap(),
            serde_json::to_value(yes.observe(p.other())).unwrap()
        );
        checked(&mut no, Action::DeclineEffectPayment);
        checked(&mut yes, Action::DeclineEffectPayment);
        assert_eq!(
            serde_json::to_value(no.observe(p.other())).unwrap(),
            serde_json::to_value(yes.observe(p.other())).unwrap()
        );
    }
}
#[test]
fn msh_thirst_short_or_empty_library_finishes_discard_before_empty_draw_loss() {
    for p in [Player::P0, Player::P1] {
        for n in 0..=2 {
            for protected in [false, true] {
                let mut g = setup(p);
                remaining_library(&mut g, p, n);
                if protected {
                    named(&mut g, p, "Herald of Eternal Dawn", Zone::Battlefield);
                }
                let drawn: Vec<_> = g.state.players[p.index()].library.to_vec();
                let spell = thirst(&mut g, p);
                assert_eq!(g.hand_count(p), n);
                assert!(g.state.players[p.index()].attempted_empty_draw);
                checked(&mut g, Action::DeclineEffectPayment);
                for &card in &drawn {
                    assert!(g.outcome().is_none());
                    assert_eq!(g.object_view(spell).zone, Zone::Stack);
                    checked(&mut g, Action::Discard { card });
                }
                assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
                for card in drawn {
                    assert_eq!(g.object_view(card).zone, Zone::Graveyard);
                }
                assert_eq!(g.hand_count(p), 0);
                assert_eq!(
                    g.state.players[p.index()].cards_discarded_this_turn,
                    n as u64
                );
                assert_eq!(g.outcome().is_none(), protected);
                if protected {
                    assert!(!g.state.players[p.index()].attempted_empty_draw);
                }
            }
        }
    }
}
#[test]
fn msh_thirst_replaced_artifact_discard_pays_once_through_exile_or_shuffle() {
    for p in [Player::P0, Player::P1] {
        for rip in [false, true] {
            for exile_first in [false, true] {
                let mut g = setup(p);
                let artifact = named(&mut g, p, "Darksteel Colossus", Zone::Hand);
                if rip {
                    named(&mut g, p, "Rest in Peace", Zone::Battlefield);
                }
                let spell = thirst(&mut g, p);
                checked(&mut g, Action::Discard { card: artifact });
                if rip {
                    assert!(matches!(g.state.pending, Pending::ZoneReplacement { .. }));
                    assert!(g.observe(p.other()).zone_replacement_choice.is_none());
                    assert_eq!(g.object_view(spell).zone, Zone::Stack);
                    let effect = if exile_first {
                        ZoneReplacementKind::Exile
                    } else {
                        ZoneReplacementKind::RevealShuffle
                    };
                    checked(&mut g, Action::ChooseZoneReplacement { effect });
                }
                settle(&mut g);
                assert_eq!(
                    g.object_view(artifact).zone,
                    if rip && exile_first {
                        Zone::Exile
                    } else {
                        Zone::Library
                    }
                );
                assert_eq!(g.hand_count(p), 3);
                assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
                assert!(g.state.resolving.is_none());
            }
        }
    }
}
#[test]
fn msh_thirst_fallback_discard_is_own_caused_and_replacement_preserves_two_card_batch() {
    for p in [Player::P0, Player::P1] {
        for exile in [false, true] {
            let mut g = setup(p);
            let liege = named(&mut g, p, "Wilt-Leaf Liege", Zone::Hand);
            let other = named(&mut g, p, "Gilded Lotus", Zone::Hand);
            if exile {
                named(&mut g, p, "Rest in Peace", Zone::Battlefield);
            }
            thirst(&mut g, p);
            checked(&mut g, Action::DeclineEffectPayment);
            checked(&mut g, Action::Discard { card: liege });
            assert_eq!(g.object_view(liege).zone, Zone::Hand);
            checked(&mut g, Action::Discard { card: other });
            settle(&mut g);
            let destination = if exile { Zone::Exile } else { Zone::Graveyard };
            assert_eq!(g.object_view(liege).zone, destination);
            assert_eq!(g.object_view(other).zone, destination);
            assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 2);
        }
    }
}
#[test]
fn msh_thirst_draw_and_discard_triggers_wait_until_the_entire_spell_finishes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let mystic = named(&mut g, p, "Mischievous Mystic", Zone::Battlefield);
        let mako = named(&mut g, p, "Marauding Mako", Zone::Battlefield);
        let a = named(&mut g, p, "Forest", Zone::Hand);
        let b = named(&mut g, p, "Forest", Zone::Hand);
        thirst(&mut g, p);
        assert_eq!(g.state.stack.len(), 0);
        assert_eq!(g.state.pending_triggers.len(), 1);
        assert_eq!(g.counter_count(mako, CounterKind::PlusOne), 0);
        checked(&mut g, Action::DeclineEffectPayment);
        checked(&mut g, Action::Discard { card: a });
        assert_eq!(g.state.stack.len(), 0);
        assert_eq!(g.state.pending_triggers.len(), 1);
        checked(&mut g, Action::Discard { card: b });
        settle(&mut g);
        assert_eq!(g.counter_count(mako, CounterKind::PlusOne), 2);
        assert_eq!(g.object_view(mystic).zone, Zone::Battlefield);
        assert_eq!(
            g.state
                .objects
                .iter()
                .filter(|o| o.zone == Zone::Battlefield && o.is_token())
                .count(),
            1
        );
    }
}
#[test]
fn msh_thirst_countered_paid_spell_never_draws_or_offers_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Thirst for Knowledge", Zone::Hand);
        cast(&mut g, id, None, &[]);
        checked(&mut g, Action::Pass);
        let negate = named(&mut g, p.other(), "Negate", Zone::Hand);
        cast(&mut g, negate, None, &[Target::Spell(id)]);
        settle(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 0);
        assert!(g.state.resolving.is_none());
    }
}

#[test]
fn msh_thirst_failed_draw_still_allows_artifact_payment_before_next_sba() {
    for p in [Player::P0, Player::P1] {
        for protected in [false, true] {
            let mut g = setup(p);
            remaining_library(&mut g, p, 0);
            if protected {
                named(&mut g, p, "Herald of Eternal Dawn", Zone::Battlefield);
            }
            let artifact = named(&mut g, p, "Gilded Lotus", Zone::Hand);
            let spell = thirst(&mut g, p);
            assert!(g.state.players[p.index()].attempted_empty_draw);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            checked(&mut g, Action::Discard { card: artifact });
            assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].cards_discarded_this_turn, 1);
            assert_eq!(g.outcome().is_none(), protected);
        }
    }
}
