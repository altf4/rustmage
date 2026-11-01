//! Focused printed outcomes; shared engine invariants live in core tests.
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
        .filter(|(_, o)| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.card == Card::from_name("@token/heartwood").unwrap()
        })
        .map(|(i, _)| i as CardId)
        .collect()
}
#[test]
fn prepared_spell_uses_both_hybrid_colors_and_creates_exact_heartwood() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn cached_funding_keeps_hand_and_graveyard_origins_distinct() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
