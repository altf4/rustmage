//! Gated candidate tests; complete physical admissions remain false until validated.
use super::big_helpers::{checked, settle, setup};
use super::*;

// Keep production admission closed while exercising the directly authored candidate.
fn candidate(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = put(g, p, "Forest", Zone::Hand);
    g.state.objects[id as usize].card = Card::from_name(name).unwrap();
    id
}
fn copy(g: &Game, source: CardId) -> ObjectRef {
    g.state
        .preparations
        .as_ref()
        .unwrap()
        .iter()
        .find(|r| r.permanent == g.object_ref(source))
        .unwrap()
        .spell_copy
}
fn finish_announcement(g: &mut Game) {
    for _ in 0..20 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseSpellCost { .. }
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::ChooseManaLotSpend { .. }
                )
            })
            .expect("supported Soul Tether announcement");
        checked(g, action);
    }
    panic!("Soul Tether announcement did not finish");
}
fn cast_creature(g: &mut Game, card: CardId) {
    checked(g, Action::AnnounceCast { card });
    finish_announcement(g);
}
fn cast_prepared(g: &mut Game, spell: ObjectRef) {
    checked(g, Action::AnnounceExile { card: spell.card });
    finish_announcement(g);
}

fn tokens(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.controller == p && o.card == Card(5438))
        .map(|(i, _)| i as CardId)
        .collect()
}
#[test]
fn prepared_spell_uses_both_hybrid_colors_and_creates_exact_heartwood() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Heartwood Crafter // Soul Tether",
            "Konstrari Improviser // Soul Tether",
        ] {
            for green in [false, true] {
                let mut g = setup(p);
                let source = candidate(&mut g, p, name);
                cast_creature(&mut g, source);
                settle(&mut g);
                assert!(g.is_prepared(g.object_ref(source)));
                let spell = copy(&g, source);
                g.state.players[p.index()].mana = if green {
                    [0, 0, 0, 0, 1, 2]
                } else {
                    [0, 0, 0, 1, 0, 2]
                };
                cast_prepared(&mut g, spell);
                assert!(!g.is_prepared(g.object_ref(source)));
                settle(&mut g);
                let created = tokens(&g, p);
                assert_eq!(created.len(), 1);
                let view = g.object_view(created[0]);
                assert_eq!(view.kinds, 16);
                assert_eq!(view.colors, 24);
                assert!(view.has_subtype("Heartwood"));
                assert!(!view.tapped);
                let actions = g.legal_actions();
                for production in [[0, 0, 0, 1, 0, 0], [0, 0, 0, 0, 1, 0]] {
                    assert!(actions.iter().any(|a| matches!(a, Action::ActivateMana { card, production: actual, .. } if *card == created[0] && *actual == production)));
                }
            }
        }
    }
}
#[test]
fn hybrid_creature_fronts_accept_red_or_green() {
    for p in [Player::P0, Player::P1] {
        for (name, generic) in [
            ("Konstrari Improviser // Soul Tether", 1),
            ("Woodwork Prodigy // Soul Tether", 2),
        ] {
            for green in [false, true] {
                let mut g = setup(p);
                let source = candidate(&mut g, p, name);
                g.state.players[p.index()].mana = if green {
                    [0, 0, 0, 0, 1, generic]
                } else {
                    [0, 0, 0, 1, 0, generic]
                };
                cast_creature(&mut g, source);
                settle(&mut g);
                assert!(g.on_battlefield(source));
            }
        }
    }
}
#[test]
fn crafter_restricted_mana_rejects_hand_cast_but_funds_prepared_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = candidate(&mut g, p, "Heartwood Crafter // Soul Tether");
        cast_creature(&mut g, source);
        settle(&mut g);
        let hand_spell = put(&mut g, p, "Adventuring Gear", Zone::Hand);
        // Start from a later turn in which the creature may tap.
        g.state.objects[source as usize].summoning_sick = false;
        let prepared = copy(&g, source);
        g.state.players[p.index()].mana = [0; 6];
        // Speculative funding must not treat the untapped producer as usable
        // for this hand cast, even before its restricted lot is produced.
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: hand_spell })
        );
        let mut funding = g.clone();
        funding.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        assert!(funding.legal_actions().contains(&Action::AnnounceExile {
            card: prepared.card
        }));
        let mana = g.legal_actions().into_iter().find(|a| matches!(a, Action::ActivateMana { card, production: [0,0,0,0,0,1], .. } if *card == source)).unwrap();
        checked(&mut g, mana);
        for viewer in [p, p.other()] {
            assert!(
                g.observe(viewer).mana_lots.iter().any(|lot| lot.player == p
                    && lot.restriction == ManaRestrictionView::ExceptCastFromHand)
            );
        }
        assert_eq!(g.available_mana(p, mana_lots::Purpose::Other)[5], 1);
        // If a later permission lets this player cast from another player's
        // hand, the restriction concerns the mana owner's hand only.
        let foreign = put(&mut g, p.other(), "Adventuring Gear", Zone::Hand);
        let foreign_purpose =
            g.spell_mana_purpose_for_card(p, foreign, g.state.objects[foreign as usize].card);
        assert_eq!(g.available_mana(p, foreign_purpose)[5], 1);
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: hand_spell })
        );
        let revision = g.revision();
        let snapshot = g.snapshot();
        assert_eq!(
            g.apply(p, Action::AnnounceCast { card: hand_spell }),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.revision(), revision);
        assert_eq!(g.snapshot(), snapshot);
        // One restricted colorless lot, one ordinary colorless and green pay the copy.
        g.state.players[p.index()].mana[5] += 1;
        g.state.players[p.index()].mana[4] += 1;
        cast_prepared(&mut g, prepared);
        settle(&mut g);
        assert_eq!(tokens(&g, p).len(), 1);
    }
}
#[test]
fn cached_funding_keeps_hand_and_graveyard_origins_distinct() {
    for p in [Player::P0, Player::P1] {
        for hand_first in [false, true] {
            let mut g = setup(p);
            let source = candidate(&mut g, p, "Heartwood Crafter // Soul Tether");
            cast_creature(&mut g, source);
            settle(&mut g);
            g.state.objects[source as usize].summoning_sick = false;
            let (hand, grave) = if hand_first {
                let hand = put(&mut g, p, "Revenge of the Rats", Zone::Hand);
                let grave = put(&mut g, p, "Revenge of the Rats", Zone::Graveyard);
                (hand, grave)
            } else {
                let grave = put(&mut g, p, "Revenge of the Rats", Zone::Graveyard);
                let hand = put(&mut g, p, "Revenge of the Rats", Zone::Hand);
                (hand, grave)
            };
            // Both faces cost {2}{B}{B}; only the graveyard cast may use the
            // producer's second generic mana.
            g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
            let actions = g.legal_actions();
            assert!(!actions.contains(&Action::AnnounceCast { card: hand }));
            assert!(actions.contains(&Action::AnnounceFlashback { card: grave }));
        }
    }
}
#[test]
fn crafter_mana_can_pay_activated_ability_costs() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = candidate(&mut g, p, "Heartwood Crafter // Soul Tether");
        cast_creature(&mut g, source);
        settle(&mut g);
        g.state.objects[source as usize].summoning_sick = false;
        let gear = put(&mut g, p, "Adventuring Gear", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        let mana = g.legal_actions().into_iter().find(|a| matches!(a, Action::ActivateMana { card, production: [0, 0, 0, 0, 0, 1], .. } if *card == source)).unwrap();
        checked(&mut g, mana);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: gear,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(source),
            },
        );
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        settle(&mut g);
        assert_eq!(
            g.state.objects[gear as usize].attached_to,
            Some(g.object_ref(source))
        );
    }
}
#[test]
fn woodwork_upkeep_intervening_if_checks_both_event_and_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = candidate(&mut g, p, "Woodwork Prodigy // Soul Tether");
        cast_creature(&mut g, source);
        settle(&mut g);
        assert!(!g.is_prepared(g.object_ref(source)));
        g.state.active = p.other();
        g.begin_step(Step::Upkeep);
        g.give_priority();
        g.prepare_triggers();
        settle(&mut g);
        assert!(!g.is_prepared(g.object_ref(source)));
        g.state.active = p;
        g.begin_step(Step::Upkeep);
        g.give_priority();
        g.prepare_triggers();
        settle(&mut g);
        assert!(g.is_prepared(g.object_ref(source)));
        let original = copy(&g, source);
        // A prepared source must not queue another upkeep trigger.
        g.begin_step(Step::Upkeep);
        assert!(g.state.pending_triggers.is_empty());
        g.give_priority();
        g.prepare_triggers();
        settle(&mut g);
        assert_eq!(copy(&g, source), original);
        // Unprepare, queue the trigger, then prepare before it resolves.
        g.forget_prepared(g.object_ref(source));
        g.begin_step(Step::Upkeep);
        assert_eq!(g.state.pending_triggers.len(), 1);
        let item = g.state.pending_triggers[0].clone();
        let condition = g.state.objects[source as usize].card.triggers()[0].condition;
        assert!(g.trigger_item_guards_hold(&item, condition));
        let mut departed = g.clone();
        departed.move_object(source, Zone::Graveyard);
        assert!(!departed.trigger_item_guards_hold(&item, condition));
        g.give_priority();
        g.prepare_triggers();
        g.prepare_permanent(g.object_ref(source));
        assert!(!g.trigger_item_guards_hold(&item, condition));
        let replacement = copy(&g, source);
        settle(&mut g);
        assert_eq!(copy(&g, source), replacement);
    }
}

#[test]
fn candidate_fronts_have_correct_names_costs_and_remain_gated() {
    for (identity, front, cost, mana_value) in [
        (
            "Heartwood Crafter // Soul Tether",
            "Heartwood Crafter",
            "{G}",
            1,
        ),
        (
            "Konstrari Improviser // Soul Tether",
            "Konstrari Improviser",
            "{1}{R/G}",
            2,
        ),
        (
            "Woodwork Prodigy // Soul Tether",
            "Woodwork Prodigy",
            "{2}{R/G}",
            3,
        ),
    ] {
        let card = Card::from_name(identity).unwrap();
        assert_eq!(Card::from_name(front), Some(card));
        assert_eq!(card.name(), front);
        assert_eq!(card.info().mana_cost, cost);
        assert_eq!(card.fixed_mana_value(), mana_value);
        assert!(!card.is_supported());
    }
}
