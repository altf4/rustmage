//! Whole printed definitions owned by the S–Z TDM partition.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

fn reach(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..40 {
        if ready(g) {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        act(g, Action::Pass);
    }
    panic!("40-priority-action whole-card ceiling");
}
fn harmonize(g: &mut Game, spell: CardId, helper: Option<CardId>, target: Option<CardId>) {
    act(g, Action::AnnounceHarmonize { card: spell });
    let object = helper.map(|id| g.object_ref(id));
    act(g, Action::ChooseHarmonizer { object });
    if let Some(target) = target {
        act(
            g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
    }
    pay(g);
    if let Some(object) = object {
        let clause = g
            .state
            .announcing
            .as_ref()
            .unwrap()
            .cast_costs
            .as_ref()
            .unwrap()
            .clauses
            .iter()
            .position(|row| row.clause == CastCostClause::Harmonize(object) && !row.paid)
            .expect("selected actual Harmonize tap cost");
        act(
            g,
            Action::ChooseCastCost {
                clause: clause as u16,
            },
        );
    }
}

#[test]
fn harmonize_unending_whisper_draws_actual_top_card_for_u_or_five_u_without_a_helper() {
    for p in [Player::P0, Player::P1] {
        for graveyard in [false, true] {
            let mut g = scenario(p);
            let top = put(&mut g, p, "Bear Cub", Zone::Library);
            let spell = put(
                &mut g,
                p,
                "Unending Whisper",
                if graveyard {
                    Zone::Graveyard
                } else {
                    Zone::Hand
                },
            );
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, if graveyard { 5 } else { 0 }];
            let before = g.hand_count(p);
            if graveyard {
                harmonize(&mut g, spell, None, None);
            } else {
                cast(&mut g, spell, None);
            }
            settle(&mut g);
            assert_eq!(g.object_view(top).zone, Zone::Hand);
            assert_eq!(g.hand_count(p), before + usize::from(graveyard));
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.object_view(spell).zone,
                if graveyard {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
            assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == top));
        }
    }
}

#[test]
fn harmonize_unending_whisper_uses_current_power_but_never_pays_the_blue_symbol() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let helper = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.objects[helper as usize].growth.power = 3;
        g.state.objects[helper as usize].summoning_sick = true;
        let spell = put(&mut g, p, "Unending Whisper", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 20];
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceHarmonize { card: spell })
        );
        reject(&mut g, Action::AnnounceHarmonize { card: spell });
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        harmonize(&mut g, spell, Some(helper), None);
        assert!(g.object_view(helper).tapped);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 1);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn harmonize_ureni_rebuff_returns_a_stolen_creature_to_its_owner_from_both_routes() {
    for p in [Player::P0, Player::P1] {
        for graveyard in [false, true] {
            let mut g = scenario(p);
            let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.change_controller(g.object_ref(victim), p);
            let helper = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let spell = put(
                &mut g,
                p,
                "Ureni's Rebuff",
                if graveyard {
                    Zone::Graveyard
                } else {
                    Zone::Hand
                },
            );
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, if graveyard { 3 } else { 1 }];
            if graveyard {
                harmonize(&mut g, spell, Some(helper), Some(victim));
            } else {
                cast(&mut g, spell, Some(Target::Card(victim)));
            }
            settle(&mut g);
            assert_eq!(g.object_view(victim).zone, Zone::Hand);
            assert_eq!(g.object_view(victim).owner, p.other());
            assert_eq!(g.hand_count(p.other()), 1);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.object_view(spell).zone,
                if graveyard {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}

#[test]
fn harmonize_ureni_rebuff_fizzles_on_departed_incarnation_and_still_exiles() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Ureni's Rebuff", Zone::Graveyard);
        harmonize(&mut g, spell, None, Some(victim));
        let old = g.object_ref(victim);
        g.move_object(victim, Zone::Graveyard);
        g.move_object(victim, Zone::Battlefield);
        assert_ne!(g.object_ref(victim), old);
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
    }
}

#[test]
fn harmonize_wild_ride_grants_growth_and_haste_only_until_cleanup_from_both_routes() {
    for p in [Player::P0, Player::P1] {
        for graveyard in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(
                &mut g,
                p,
                "Wild Ride",
                if graveyard {
                    Zone::Graveyard
                } else {
                    Zone::Hand
                },
            );
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, if graveyard { 4 } else { 0 }];
            if graveyard {
                harmonize(&mut g, spell, None, Some(target));
            } else {
                cast(&mut g, spell, Some(Target::Card(target)));
            }
            settle(&mut g);
            assert_eq!(stats(&g, target), (5, 2));
            assert!(g.keywords(target).contains(Keywords::HASTE));
            assert_eq!(
                g.object_view(spell).zone,
                if graveyard {
                    Zone::Exile
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            g.finish_cleanup();
            assert_eq!(stats(&g, target), (2, 2));
            assert!(!g.keywords(target).contains(Keywords::HASTE));
        }
    }
}

#[test]
fn harmonize_all_four_require_sorcery_timing_and_exile_when_countered() {
    for p in [Player::P0, Player::P1] {
        for name in [
            "Unending Whisper",
            "Ureni's Rebuff",
            "Wild Ride",
            "Winternight Stories",
        ] {
            let mut g = scenario(p);
            let target = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, name, Zone::Graveyard);
            g.state.active = p.other();
            assert!(
                !g.legal_actions()
                    .contains(&Action::AnnounceHarmonize { card: spell })
            );
            reject(&mut g, Action::AnnounceHarmonize { card: spell });
            g.state.active = p;
            harmonize(
                &mut g,
                spell,
                None,
                matches!(name, "Ureni's Rebuff" | "Wild Ride").then_some(target),
            );
            act(&mut g, Action::Pass);
            let cancel = put(&mut g, p.other(), "Cancel", Zone::Hand);
            cast(&mut g, cancel, Some(Target::Spell(spell)));
            settle(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Exile, "{name}");
            assert_eq!(g.hand_count(p), 0, "countered {name} did not draw");
            assert_eq!(stats(&g, target), (2, 2));
            assert!(!g.keywords(target).contains(Keywords::HASTE));
        }
    }
}

#[test]
fn harmonize_winternight_draws_first_then_discards_one_creature_or_two_cards_privately() {
    for p in [Player::P0, Player::P1] {
        for creature_route in [false, true] {
            let mut g = scenario(p);
            let creature = put(&mut g, p, "Bear Cub", Zone::Library);
            put(&mut g, p, "Island", Zone::Library);
            put(&mut g, p, "Forest", Zone::Library);
            let spell = put(&mut g, p, "Winternight Stories", Zone::Hand);
            g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
            cast(&mut g, spell, None);
            reach(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectChoice { .. })
            });
            assert_eq!(g.hand_count(p), 3);
            assert_eq!(g.object_view(creature).zone, Zone::Hand);
            assert!(g.observe(p.other()).effect_choice.is_none());
            let noncreature = g
                .state
                .objects
                .iter()
                .enumerate()
                .find_map(|(id, o)| {
                    (o.zone == Zone::Hand && o.owner == p && o.card.is_land())
                        .then_some(id as CardId)
                })
                .unwrap();
            reject(&mut g, Action::Discard { card: noncreature });
            if creature_route {
                act(&mut g, Action::Discard { card: creature });
            } else {
                act(&mut g, Action::DeclineEffectPayment);
                assert!(matches!(
                    g.state.pending,
                    Pending::EffectDiscard { remaining: 2, .. }
                ));
                act(&mut g, Action::Discard { card: creature });
                act(&mut g, Action::Discard { card: noncreature });
            }
            settle(&mut g);
            assert_eq!(g.hand_count(p), if creature_route { 2 } else { 1 });
            assert_eq!(g.object_view(creature).zone, Zone::Graveyard);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}

#[test]
fn harmonize_winternight_discard_choice_is_owned_raw_index_restorable_and_parent_independent() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let helper = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let creature = put(&mut g, p, "Bear Cub", Zone::Library);
        put(&mut g, p, "Island", Zone::Library);
        put(&mut g, p, "Forest", Zone::Library);
        let spell = put(&mut g, p, "Winternight Stories", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        harmonize(&mut g, spell, Some(helper), None);
        reach(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectChoice { .. })
        });
        let action = Action::Discard { card: creature };
        let revision = g.revision();
        let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
        let root = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        let mut sibling = g.clone();
        drop(g);
        raw.apply(p, action.clone()).unwrap();
        indexed.apply_index(p, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert!(raw.observe(p.other()).effect_choice.is_none());
        assert_eq!(
            serde_json::to_value(raw.observe(p.other())).unwrap(),
            serde_json::to_value(indexed.observe(p.other())).unwrap()
        );
        act(&mut sibling, Action::DeclineEffectPayment);
        act(&mut sibling, Action::Discard { card: creature });
        choose(&mut sibling, |a| matches!(a, Action::Discard { .. }));
        settle(&mut raw);
        settle(&mut indexed);
        settle(&mut sibling);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.hand_count(p), 2);
        assert_eq!(sibling.hand_count(p), 1);
        assert_eq!(raw.object_view(spell).zone, Zone::Exile);
        assert!(raw.object_view(helper).tapped);
        indexed.restore(&root).unwrap();
        assert_eq!(
            indexed.apply_index(p, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, action);
        settle(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}
