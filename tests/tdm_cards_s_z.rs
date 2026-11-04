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

// Gated definitions compile normally. Substitute the physical card while it is
// still in hand, before any ordinary play/cast or source-registration event.
fn actual_card(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).expect("catalogue identity");
    if card.is_supported() {
        return put(g, p, name, zone);
    }
    let id = put(g, p, "Bear Cub", Zone::Hand);
    g.state.objects[id as usize].card = card;
    if zone != Zone::Hand {
        g.move_object(id, zone);
    }
    id
}
fn mana_action(g: &Game, id: CardId, color: usize) -> Action {
    let mut production = [0; 6];
    production[color] = 1;
    g.legal_actions()
        .into_iter()
        .find(|a| {
            matches!(a, Action::ActivateMana { card, production: made, .. }
            if *card == id && *made == production)
                || matches!(a, Action::TapMana { card } if *card == id && color == 2)
        })
        .unwrap_or_else(|| panic!("no mana color {color} for {}", g.object_view(id).name()))
}
fn activated(g: &mut Game, id: CardId, ability: u16, target: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(g);
}

#[test]
fn mana_citadel_enters_tapped_and_offers_exactly_white_black_green_with_land_timing() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Sandsteppe Citadel", Zone::Hand);
        let second = put(&mut g, p, "Forest", Zone::Hand);
        let mut off_turn = g.clone();
        off_turn.state.active = p.other();
        reject(&mut off_turn, Action::PlayLand { card: land });
        let mut upkeep = g.clone();
        upkeep.state.step = Step::Upkeep;
        reject(&mut upkeep, Action::PlayLand { card: land });
        act(&mut g, Action::PlayLand { card: land });
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        assert!(g.object_view(land).tapped);
        assert_eq!(g.state.players[p.index()].life, 20);
        reject(&mut g, Action::PlayLand { card: second });
        g.state.players[p.index()].mana = [0; 6];
        assert!(!g.legal_actions().iter().any(|a| matches!(a,
            Action::ActivateMana { card, .. } if *card == land)));
        g.state.objects[land as usize].tapped = false;
        let offered: Vec<_> = g
            .legal_actions()
            .into_iter()
            .filter_map(|a| match a {
                Action::ActivateMana {
                    card, production, ..
                } if card == land => Some(production),
                _ => None,
            })
            .collect();
        assert_eq!(offered.len(), 3);
        for color in [0, 2, 4] {
            let mut child = g.clone();
            let action = mana_action(&child, land, color);
            act(&mut child, action.clone());
            let mut expected = [0; 6];
            expected[color] = 1;
            assert_eq!(child.state.players[p.index()].mana, expected);
            assert!(child.object_view(land).tapped);
            assert!(child.state.stack.is_empty());
            reject(&mut child, action);
        }
    }
}

#[test]
fn mana_devotees_filter_one_mana_without_tapping_even_when_summoning_sick_and_only_once_each_turn()
{
    for p in [Player::P0, Player::P1] {
        for (name, colors, expected_stats, keyword) in [
            (
                "Sultai Devotee",
                &[1, 2, 4][..],
                (2, 1),
                Keywords::DEATHTOUCH,
            ),
            ("Temur Devotee", &[1, 3, 4][..], (3, 3), Keywords::DEFENDER),
        ] {
            let mut g = scenario(p);
            let id = put(&mut g, p, name, Zone::Hand);
            cast(&mut g, id, None);
            settle(&mut g);
            assert_eq!(stats(&g, id), expected_stats);
            assert!(g.keywords(id).contains(keyword));
            assert!(g.object_view(id).summoning_sick);
            g.state.players[p.index()].mana = [0; 6];
            assert!(!g.legal_actions().iter().any(|a| matches!(a,
                Action::ActivateMana { card, .. } if *card == id)));
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            let actions = g.legal_actions();
            eprintln!("{name} mana offers: {actions:?}");
            let mut offered: Vec<_> = actions
                .into_iter()
                .filter_map(|a| match a {
                    Action::ActivateMana {
                        card, production, ..
                    } if card == id => Some(production),
                    _ => None,
                })
                .collect();
            offered.sort_unstable();
            offered.dedup();
            assert_eq!(
                offered.iter().filter(|&&m| m == [0; 6]).count(),
                1,
                "zero is the deferred color-choice route, not a fourth color"
            );
            offered.retain(|&m| m != [0; 6]);
            let mut expected_outputs: Vec<_> = colors
                .iter()
                .map(|&color| {
                    let mut production = [0; 6];
                    production[color] = 1;
                    production
                })
                .collect();
            expected_outputs.sort_unstable();
            assert_eq!(offered, expected_outputs);
            for &color in colors {
                let mut child = g.clone();
                let action = mana_action(&child, id, color);
                act(&mut child, action.clone());
                let mut expected = [0; 6];
                expected[color] = 1;
                assert_eq!(child.state.players[p.index()].mana, expected);
                assert!(!child.object_view(id).tapped);
                assert!(child.state.stack.is_empty());
                reject(&mut child, action);
            }
            let deferred_action = g.legal_actions().into_iter().find(|a| matches!(a,
                Action::ActivateMana { card, production, .. } if *card == id && *production == [0; 6]
            )).expect("ordinary deferred color choice");
            act(&mut g, deferred_action.clone());
            assert_eq!(
                g.state.players[p.index()].mana,
                [0; 6],
                "generic cost paid before color choice"
            );
            let mut choices: Vec<_> = g
                .legal_actions()
                .into_iter()
                .filter_map(|a| match a {
                    Action::ChooseMana { production } => Some(production),
                    _ => None,
                })
                .collect();
            choices.sort_unstable();
            assert_eq!(choices, expected_outputs);
            reject(
                &mut g,
                Action::ChooseMana {
                    production: [1, 0, 0, 0, 0, 0],
                },
            );
            act(
                &mut g,
                Action::ChooseMana {
                    production: [0, 1, 0, 0, 0, 0],
                },
            );
            assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 0, 0, 0]);
            assert!(!g.object_view(id).tapped);
            assert!(g.state.stack.is_empty());
            reject(&mut g, deferred_action);
        }
    }
}

#[test]
fn mana_devotee_use_survives_control_change_and_resets_on_the_next_actual_turn() {
    for p in [Player::P0, Player::P1] {
        for name in ["Sultai Devotee", "Temur Devotee"] {
            let mut g = scenario(p);
            let id = put(&mut g, p, name, Zone::Battlefield);
            let action = mana_action(&g, id, 1);
            act(&mut g, action);
            g.change_controller(g.object_ref(id), p.other());
            act(&mut g, Action::Pass);
            assert_eq!(g.actor(), Some(p.other()));
            assert!(!g.legal_actions().iter().any(|a| matches!(a,
                Action::ActivateMana { card, .. } if *card == id)));
            g.state.step = Step::Main2;
            reach(&mut g, |g| {
                g.state.turn == 4 && g.state.step == Step::Main1 && g.actor() == Some(p.other())
            });
            g.state.players[p.other().index()].mana = [0, 0, 0, 0, 0, 1];
            let action = mana_action(&g, id, 1);
            act(&mut g, action);
            assert!(!g.object_view(id).tapped);
        }
    }
}

#[test]
fn mana_sunset_casts_as_three_one_and_taps_for_red_but_cannot_pay_both_tap_costs() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = actual_card(&mut g, p, "Sunset Strikemaster", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
        cast(&mut g, id, None);
        settle(&mut g);
        assert_eq!(stats(&g, id), (3, 1));
        assert!(g.object_view(id).summoning_sick);
        assert!(!g.legal_actions().iter().any(|a| matches!(a,
            Action::ActivateMana { card, .. } | Action::AnnounceActivate { card, .. } if *card == id)));
        g.state.objects[id as usize].summoning_sick = false;
        let action = mana_action(&g, id, 3);
        act(&mut g, action);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 1, 0, 0]);
        assert!(g.state.stack.is_empty());
        assert!(g.object_view(id).tapped);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
    }
}

#[test]
fn mana_sunset_pays_two_red_tap_sacrifice_and_deals_six_only_to_a_flying_creature() {
    for p in [Player::P0, Player::P1] {
        for owner in [p, p.other()] {
            let mut g = scenario(p);
            let id = actual_card(&mut g, p, "Sunset Strikemaster", Zone::Battlefield);
            g.state.objects[id as usize].summoning_sick = false;
            let bird = put(&mut g, owner, "Serra Angel", Zone::Battlefield);
            g.state.objects[bird as usize].growth.toughness = 3;
            let ground = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            let start = Action::AnnounceActivate {
                card: id,
                ability: 1,
                counter_payment: Counters::default(),
            };
            reject(&mut g, start.clone());
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
            // No sorcery restriction: announce at upkeep with priority.
            g.state.step = Step::Upkeep;
            act(&mut g, start);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(ground),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bird),
                },
            );
            pay(&mut g);
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.state.stack.len(), 1);
            settle(&mut g);
            assert_eq!(g.object_view(bird).zone, Zone::Battlefield);
            assert_eq!(g.object_view(bird).damage, 6);
            assert_eq!(g.object_view(ground).damage, 0);
        }
    }
}

#[test]
fn mana_sunset_target_loses_flying_or_changes_incarnation_before_resolution() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = scenario(p);
            let id = actual_card(&mut g, p, "Sunset Strikemaster", Zone::Battlefield);
            g.state.objects[id as usize].summoning_sick = false;
            let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[target as usize].gained_keywords = Keywords::FLYING;
            activated(&mut g, id, 1, target);
            if blink {
                g.move_object(target, Zone::Hand);
                g.move_object(target, Zone::Battlefield);
            } else {
                g.state.objects[target as usize].gained_keywords = Keywords::NONE;
            }
            settle(&mut g);
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            assert_eq!(g.object_view(target).damage, 0);
        }
    }
}

#[test]
fn mana_devotee_color_choice_is_owned_raw_index_restorable_and_does_not_expose_a_private_hand() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let id = put(&mut g, p, "Sultai Devotee", Zone::Battlefield);
        let private = put(&mut g, p, "Cancel", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        let action = mana_action(&g, id, 2);
        let revision = g.revision();
        let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
        let root = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        let mut sibling = g.clone();
        drop(g);
        raw.apply(p, action.clone()).unwrap();
        indexed.apply_index(p, revision, index).unwrap();
        let different = mana_action(&sibling, id, 4);
        act(&mut sibling, different);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.state.players[p.index()].mana, [0, 0, 1, 0, 0, 0]);
        assert_eq!(sibling.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
        assert!(
            !raw.observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == private)
        );
        assert_eq!(
            serde_json::to_value(raw.observe(p.other())).unwrap(),
            serde_json::to_value(indexed.observe(p.other())).unwrap()
        );
        indexed.restore(&root).unwrap();
        assert_eq!(
            indexed.apply_index(p, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, action);
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}

fn trigger_target(g: &mut Game, target: Target) {
    for _ in 0..40 {
        let action = Action::ChooseTarget { target };
        if g.legal_actions().contains(&action) {
            act(g, action);
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(g, Action::FinishTargets);
            }
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("40-action entry-target ceiling");
}
fn spirits(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(id as CardId, "Spirit"))
            .then_some(id as CardId)
        })
        .collect()
}
fn attack_one(g: &mut Game, id: CardId) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.priority = g.state.active;
    act(g, Action::Attack { card: id });
    act(g, Action::FinishAttackers);
}

#[test]
fn creatures_sagu_pummeler_casts_four_four_reach_and_renews_two_plus_one_and_one_reach_counter() {
    for p in [Player::P0, Player::P1] {
        for owner in [p, p.other()] {
            let mut g = scenario(p);
            let source = actual_card(&mut g, p, "Sagu Pummeler", Zone::Hand);
            let target = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 3];
            cast(&mut g, source, None);
            settle(&mut g);
            assert_eq!(stats(&g, source), (4, 4));
            assert!(g.keywords(source).contains(Keywords::REACH));
            g.move_object(source, Zone::Graveyard);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 4];
            activated(&mut g, source, 0, target);
            assert_eq!(g.object_view(source).zone, Zone::Exile);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(g.counter_count(target, CounterKind::PlusOne), 2);
            assert_eq!(g.counter_count(target, CounterKind::Reach), 1);
            assert_eq!(stats(&g, target), (4, 4));
            assert!(g.keywords(target).contains(Keywords::REACH));
            g.finish_cleanup();
            assert!(g.keywords(target).contains(Keywords::REACH));
            assert_eq!(g.counter_count(target, CounterKind::Reach), 1);
        }
    }
}

#[test]
fn creatures_sagu_renew_rejects_wrong_zone_timing_cost_target_and_never_rebinds_blinked_targets() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = actual_card(&mut g, p, "Sagu Pummeler", Zone::Hand);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let artifact = put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
        let start = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, start.clone());
        g.move_object(source, Zone::Graveyard);
        let mut upkeep = g.clone();
        upkeep.state.step = Step::Upkeep;
        reject(&mut upkeep, start.clone());
        let mut off_turn = g.clone();
        off_turn.state.active = p.other();
        reject(&mut off_turn, start.clone());
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5];
        reject(&mut g, start.clone());
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 4];
        act(&mut g, start);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        g.move_object(target, Zone::Hand);
        g.move_object(target, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Exile);
        assert_eq!(g.counter_count(target, CounterKind::PlusOne), 0);
        assert_eq!(g.counter_count(target, CounterKind::Reach), 0);
    }
}

#[test]
fn creatures_packbeast_affinity_counts_only_controlled_creatures_then_draws_the_actual_top_card() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        put(&mut g, p, "Bear Cub", Zone::Battlefield);
        put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        put(&mut g, p, "Short Bow", Zone::Battlefield);
        let top = put(&mut g, p, "Cancel", Zone::Library);
        let source = put(&mut g, p, "Salt Road Packbeast", Zone::Hand);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 2];
        reject(&mut g, Action::AnnounceCast { card: source });
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 3];
        cast(&mut g, source, None);
        settle(&mut g);
        assert_eq!(stats(&g, source), (4, 3));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(top).zone, Zone::Hand);
        assert_eq!(g.hand_count(p), 1);
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == top));
    }
}

#[test]
fn creatures_outrider_and_surveyor_choose_exact_endure_sizes_and_surveyor_loses_one_on_attack() {
    for p in [Player::P0, Player::P1] {
        for (name, amount, base, keyword) in [
            ("Sandskitter Outrider", 2, (2, 1), Keywords::MENACE),
            ("Sinkhole Surveyor", 1, (1, 3), Keywords::FLYING),
        ] {
            for counters in [false, true] {
                let mut g = scenario(p);
                let id = put(&mut g, p, name, Zone::Hand);
                cast(&mut g, id, None);
                if name == "Sinkhole Surveyor" {
                    settle(&mut g);
                    assert_eq!(g.state.players[p.index()].life, 20);
                    g.state.objects[id as usize].summoning_sick = false;
                    attack_one(&mut g, id);
                }
                reach(&mut g, |g| {
                    matches!(g.state.pending, Pending::Endure { .. })
                });
                assert_eq!(stats(&g, id), base);
                assert!(g.keywords(id).contains(keyword));
                assert_eq!(
                    g.state.players[p.index()].life,
                    if name == "Sinkhole Surveyor" { 19 } else { 20 }
                );
                assert_eq!(g.state.players[p.other().index()].life, 20);
                act(
                    &mut g,
                    Action::ChooseEndure {
                        put_counters: counters,
                    },
                );
                settle(&mut g);
                assert_eq!(
                    g.counter_count(id, CounterKind::PlusOne),
                    if counters { amount } else { 0 }
                );
                let made = spirits(&g, p);
                assert_eq!(made.len(), usize::from(!counters));
                if let Some(&spirit) = made.first() {
                    assert_eq!(stats(&g, spirit), (amount as i128, amount as i128));
                    assert_eq!(g.object_view(spirit).colors, 1);
                    assert!(!g.keywords(spirit).contains(Keywords::FLYING));
                }
            }
        }
    }
}

#[test]
fn creatures_outrider_departure_still_creates_exact_spirit_and_endure_branches_are_owned() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Sandskitter Outrider", Zone::Hand);
        cast(&mut g, source, None);
        reach(&mut g, |g| {
            matches!(g.state.pending, Pending::Endure { .. })
        });
        let saved = g.snapshot();
        let revision = g.revision();
        let action = Action::ChooseEndure {
            put_counters: false,
        };
        let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        let mut sibling = g.clone();
        drop(g);
        raw.apply(p, action.clone()).unwrap();
        indexed.apply_index(p, revision, index).unwrap();
        act(&mut sibling, Action::ChooseEndure { put_counters: true });
        for child in [&mut raw, &mut indexed, &mut sibling] {
            settle(child);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(spirits(&raw, p).len(), 1);
        assert_eq!(stats(&sibling, source), (4, 3));
        let mut departed = scenario(p);
        let left = put(&mut departed, p, "Sandskitter Outrider", Zone::Hand);
        cast(&mut departed, left, None);
        reach(&mut departed, |g| {
            g.object_view(left).zone == Zone::Battlefield
        });
        // Depart at the priority boundary before the trigger resolves. No
        // player can interleave an action inside the later Endure choice.
        departed.move_object(left, Zone::Graveyard);
        settle(&mut departed);
        assert_eq!(spirits(&departed, p).len(), 1);
        assert_eq!(stats(&departed, spirits(&departed, p)[0]), (2, 2));
        assert_eq!(departed.object_view(left).zone, Zone::Graveyard);
        indexed.restore(&saved).unwrap();
        assert_eq!(
            indexed.apply_index(p, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, action);
        settle(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}

#[test]
fn creatures_skirmish_rhino_drains_the_opponent_after_its_source_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Skirmish Rhino", Zone::Hand);
        g.state.players[p.index()].mana = [1, 0, 1, 0, 1, 0];
        cast(&mut g, source, None);
        reach(&mut g, |g| g.object_view(source).zone == Zone::Battlefield);
        assert_eq!(stats(&g, source), (3, 4));
        assert!(g.keywords(source).contains(Keywords::TRAMPLE));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.move_object(source, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 22);
        assert_eq!(g.state.players[p.other().index()].life, 18);
    }
}

#[test]
fn creatures_summit_intimidator_stops_only_the_chosen_incarnation_blocking_until_cleanup() {
    for p in [Player::P0, Player::P1] {
        for owner in [p, p.other()] {
            for blink in [false, true] {
                let mut g = scenario(p);
                let target = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
                let untouched = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
                let attacker = put(&mut g, owner.other(), "Bear Cub", Zone::Battlefield);
                let source = put(&mut g, p, "Summit Intimidator", Zone::Hand);
                cast(&mut g, source, None);
                trigger_target(&mut g, Target::Card(target));
                assert_eq!(stats(&g, source), (4, 3));
                assert!(g.keywords(source).contains(Keywords::REACH));
                if blink {
                    g.move_object(target, Zone::Hand);
                    g.move_object(target, Zone::Battlefield);
                }
                settle(&mut g);
                g.state.active = owner.other();
                assert_eq!(g.can_block_pair(target, attacker), blink);
                assert!(g.can_block_pair(untouched, attacker));
                g.finish_cleanup();
                assert!(!g.keywords(target).contains(Keywords::CANT_BLOCK));
                g.state.active = owner.other();
                assert!(g.can_block_pair(target, attacker));
            }
        }
    }
}

#[test]
fn creatures_temur_tawnyback_all_monohybrid_payments_draw_before_a_private_discard() {
    for p in [Player::P0, Player::P1] {
        for mask in 0..8u8 {
            let mut g = scenario(p);
            let top = put(&mut g, p, "Cancel", Zone::Library);
            let source = put(&mut g, p, "Temur Tawnyback", Zone::Hand);
            let mut payment = [0; 6];
            for (bit, color) in [4, 1, 3].into_iter().enumerate() {
                if mask & (1 << bit) != 0 {
                    payment[color] = 1;
                } else {
                    payment[5] += 2;
                }
            }
            g.state.players[p.index()].mana = payment;
            cast(&mut g, source, None);
            reach(&mut g, |g| {
                g.legal_actions().contains(&Action::Discard { card: top })
            });
            assert_eq!(stats(&g, source), (4, 3));
            assert_eq!(g.object_view(top).zone, Zone::Hand);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == top));
            act(&mut g, Action::Discard { card: top });
            settle(&mut g);
            assert_eq!(g.object_view(top).zone, Zone::Graveyard);
            assert_eq!(g.hand_count(p), 0);
        }
    }
}

#[test]
fn creatures_watcher_mills_either_player_exactly_two_or_a_short_library_and_gains_controller_two() {
    for p in [Player::P0, Player::P1] {
        for target in [p, p.other()] {
            for short in [false, true] {
                let mut g = scenario(p);
                if short {
                    for id in g.state.players[target.index()].library.clone() {
                        g.move_object(id, Zone::Exile);
                    }
                }
                let first = put(&mut g, target, "Bear Cub", Zone::Library);
                let second = if short {
                    None
                } else {
                    Some(put(&mut g, target, "Cancel", Zone::Library))
                };
                let source = put(&mut g, p, "Watcher of the Wayside", Zone::Hand);
                cast(&mut g, source, None);
                trigger_target(&mut g, Target::Player(target));
                settle(&mut g);
                assert_eq!(stats(&g, source), (3, 2));
                assert_eq!(g.object_view(source).kinds & 17, 17);
                assert_eq!(g.object_view(first).zone, Zone::Graveyard);
                if let Some(second) = second {
                    assert_eq!(g.object_view(second).zone, Zone::Graveyard);
                }
                assert_eq!(g.state.players[p.index()].life, 22);
                assert_eq!(g.state.players[p.other().index()].life, 20);
            }
        }
    }
}

fn modal_start(g: &mut Game, spell: CardId, mode: u16) {
    act(g, Action::AnnounceCast { card: spell });
    act(g, Action::ChooseMode { mode });
}
fn modal_cast(g: &mut Game, spell: CardId, mode: u16, targets: &[Target]) {
    modal_start(g, spell, mode);
    for &target in targets {
        act(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}
fn bolt_targets(g: &mut Game, spell: CardId, targets: &[Target]) {
    act(g, Action::AnnounceCast { card: spell });
    for &target in targets {
        act(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn allocate(g: &mut Game, amounts: &[u64]) {
    for &amount in amounts {
        act(g, Action::AllocateTargetAmount { amount });
    }
    pay(g);
}
fn pay_sacrifice(g: &mut Game, material: CardId) {
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        if matches!(g.state.pending, Pending::AdditionalCost { .. }) {
            let object = g.object_ref(material);
            act(g, Action::ChooseAdditionalCost { object });
        } else if g.legal_actions().contains(&Action::BeginOrderedCosts) {
            act(g, Action::BeginOrderedCosts);
        } else if matches!(g.state.pending, Pending::CastCosts { .. }) {
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
                .position(|row| !row.paid)
                .unwrap();
            act(
                g,
                Action::ChooseCastCost {
                    clause: clause as u16,
                },
            );
        } else {
            pay(g);
        }
    }
    panic!("eight-action printed sacrifice-cost ceiling");
}

#[test]
fn spells_sarkhan_resolve_both_modes_have_distinct_targets_and_temporary_growth() {
    for p in [Player::P0, Player::P1] {
        for owner in [p, p.other()] {
            let mut g = scenario(p);
            g.state.step = Step::Upkeep;
            let bear = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
            let source = put(&mut g, p, "Sarkhan's Resolve", Zone::Hand);
            modal_cast(&mut g, source, 0, &[Target::Card(bear)]);
            settle(&mut g);
            assert_eq!(stats(&g, bear), (5, 5));
            g.finish_cleanup();
            assert_eq!(stats(&g, bear), (2, 2));
        }
        for lose_flying in [false, true] {
            let mut g = scenario(p);
            let bird = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.state.objects[bird as usize].gained_keywords = Keywords::FLYING;
            let ground = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let source = put(&mut g, p, "Sarkhan's Resolve", Zone::Hand);
            modal_start(&mut g, source, 1);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(ground),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(bird),
                },
            );
            pay(&mut g);
            if lose_flying {
                g.state.objects[bird as usize].gained_keywords = Keywords::NONE;
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(bird).zone,
                if lose_flying {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(g.object_view(ground).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn spells_snakeskin_veil_restricts_control_and_keeps_counter_after_hexproof_expires() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = put(&mut g, p, "Snakeskin Veil", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: source });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foreign),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(stats(&g, own), (3, 3));
        assert_eq!(g.counter_count(own, CounterKind::PlusOne), 1);
        assert!(g.keywords(own).contains(Keywords::HEXPROOF));
        let mut opponent = g.clone();
        act(&mut opponent, Action::Pass);
        let shock = put(&mut opponent, p.other(), "Shock", Zone::Hand);
        act(&mut opponent, Action::AnnounceCast { card: shock });
        reject(
            &mut opponent,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        let second = put(&mut g, p, "Snakeskin Veil", Zone::Hand);
        cast(&mut g, second, Some(Target::Card(own)));
        settle(&mut g);
        assert_eq!(g.counter_count(own, CounterKind::PlusOne), 2);
        g.finish_cleanup();
        assert_eq!(stats(&g, own), (4, 4));
        assert!(!g.keywords(own).contains(Keywords::HEXPROOF));
    }
}

#[test]
fn spells_twin_bolt_requires_one_or_two_targets_and_announces_exact_positive_shares() {
    for p in [Player::P0, Player::P1] {
        for split in [false, true] {
            let mut g = scenario(p);
            let dragon = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
            let artifact = put(&mut g, p.other(), "Short Bow", Zone::Battlefield);
            let spell = actual_card(&mut g, p, "Twin Bolt", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: spell });
            reject(&mut g, Action::FinishTargets);
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(artifact),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(dragon),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(dragon),
                },
            );
            if split {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
            }
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            reject(&mut g, Action::AllocateTargetAmount { amount: 0 });
            reject(&mut g, Action::AllocateTargetAmount { amount: 3 });
            allocate(&mut g, if split { &[1, 1] } else { &[2] });
            settle(&mut g);
            assert_eq!(g.object_view(dragon).damage, if split { 1 } else { 2 });
            assert_eq!(
                g.state.players[p.other().index()].life,
                if split { 19 } else { 20 }
            );
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        }
        let mut g = scenario(p);
        let walker = put(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.set_counters(walker, Counters::of(CounterKind::Loyalty, 5));
        let spell = actual_card(&mut g, p, "Twin Bolt", Zone::Hand);
        bolt_targets(&mut g, spell, &[Target::Card(walker)]);
        allocate(&mut g, &[2]);
        settle(&mut g);
        assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 3);
    }
}

#[test]
fn spells_twin_bolt_owned_allocations_never_redistribute_a_departed_targets_share() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
        let spell = actual_card(&mut g, p, "Twin Bolt", Zone::Hand);
        bolt_targets(
            &mut g,
            spell,
            &[Target::Card(victim), Target::Player(p.other())],
        );
        let action = Action::AllocateTargetAmount { amount: 1 };
        let rev = g.revision();
        let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        let mut sibling = g.clone();
        drop(g);
        raw.apply(p, action.clone()).unwrap();
        indexed.apply_index(p, rev, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        allocate(&mut raw, &[1]);
        allocate(&mut indexed, &[1]);
        for child in [&mut raw, &mut indexed] {
            child.move_object(victim, Zone::Hand);
            child.move_object(victim, Zone::Battlefield);
            settle(child);
            assert_eq!(child.object_view(victim).damage, 0);
            assert_eq!(child.state.players[p.other().index()].life, 19);
        }
        allocate(&mut sibling, &[1, 1]);
        settle(&mut sibling);
        assert_eq!(sibling.object_view(victim).damage, 1);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        assert_eq!(
            indexed.apply_index(p, rev, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, action);
        allocate(&mut indexed, &[1]);
        indexed.move_object(victim, Zone::Hand);
        indexed.move_object(victim, Zone::Battlefield);
        settle(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}

#[test]
fn spells_wail_of_war_freezes_only_the_target_opponents_current_creatures_until_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = actual_card(&mut g, p, "Wail of War", Zone::Hand);
        modal_start(&mut g, spell, 0);
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
        pay(&mut g);
        settle(&mut g);
        assert_eq!(stats(&g, enemy), (1, 1));
        assert_eq!(stats(&g, own), (2, 2));
        let late = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        assert_eq!(stats(&g, late), (2, 2));
        g.finish_cleanup();
        assert_eq!(stats(&g, enemy), (2, 2));
    }
}

#[test]
fn spells_wail_of_war_returns_zero_one_or_two_own_creatures_with_partial_target_finality() {
    for p in [Player::P0, Player::P1] {
        for count in 0..=2 {
            for blink in [false, true] {
                let mut g = scenario(p);
                let first = put(&mut g, p, "Bear Cub", Zone::Graveyard);
                let second = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
                let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Graveyard);
                let land = put(&mut g, p, "Forest", Zone::Graveyard);
                let spell = actual_card(&mut g, p, "Wail of War", Zone::Hand);
                g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
                modal_start(&mut g, spell, 1);
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(foreign),
                    },
                );
                reject(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(land),
                    },
                );
                for &target in &[first, second][..count] {
                    act(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Card(target),
                        },
                    );
                }
                if g.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                }
                pay(&mut g);
                if blink {
                    g.move_object(first, Zone::Exile);
                    g.move_object(first, Zone::Graveyard);
                }
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                assert_eq!(
                    g.object_view(first).zone,
                    if count > 0 && !blink {
                        Zone::Hand
                    } else {
                        Zone::Graveyard
                    }
                );
                assert_eq!(
                    g.object_view(second).zone,
                    if count == 2 {
                        Zone::Hand
                    } else {
                        Zone::Graveyard
                    }
                );
                assert_eq!(g.object_view(foreign).zone, Zone::Graveyard);
                assert_eq!(g.object_view(land).zone, Zone::Graveyard);
            }
        }
    }
}

#[test]
fn spells_unsparing_boltcaster_requires_an_enemy_damaged_this_turn_and_freezes_exact_five_damage() {
    for p in [Player::P0, Player::P1] {
        for response in 0..3 {
            let mut g = scenario(p);
            let enemy = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
            g.state.objects[enemy as usize].growth.toughness = 3;
            let own = put(&mut g, p, "Shivan Dragon", Zone::Battlefield);
            let clean = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            for target in [enemy, own] {
                let shock = put(&mut g, p, "Shock", Zone::Hand);
                cast(&mut g, shock, Some(Target::Card(target)));
                settle(&mut g);
            }
            let source = actual_card(&mut g, p, "Unsparing Boltcaster", Zone::Hand);
            cast(&mut g, source, None);
            reach(&mut g, |g| {
                g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Card(enemy),
                })
            });
            assert_eq!(stats(&g, source), (3, 3));
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(clean),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(enemy),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            if response == 0 {
                g.move_object(source, Zone::Graveyard);
            }
            if response == 1 {
                g.change_controller(g.object_ref(enemy), p);
            }
            if response == 2 {
                g.move_object(enemy, Zone::Hand);
                g.move_object(enemy, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(enemy).damage,
                match response {
                    0 => 7,
                    1 => 2,
                    _ => 0,
                }
            );
            assert_eq!(g.object_view(own).damage, 2);
            assert_eq!(g.object_view(clean).damage, 0);
        }
    }
}

#[test]
fn spells_unsparing_boltcaster_can_enter_when_no_creature_is_an_eligible_entry_target() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let clean = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = actual_card(&mut g, p, "Unsparing Boltcaster", Zone::Hand);
        cast(&mut g, source, None);
        settle(&mut g);
        assert_eq!(stats(&g, source), (3, 3));
        assert_eq!(g.object_view(clean).damage, 0);
    }
}

#[test]
fn spells_seize_opportunity_boosts_zero_one_or_two_creatures_until_cleanup() {
    for p in [Player::P0, Player::P1] {
        for count in 0..=2 {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = put(&mut g, p, "Seize Opportunity", Zone::Hand);
            let all = [Target::Card(own), Target::Card(foreign)];
            modal_cast(&mut g, spell, 1, &all[..count]);
            settle(&mut g);
            assert_eq!(stats(&g, own), if count > 0 { (4, 3) } else { (2, 2) });
            assert_eq!(stats(&g, foreign), if count == 2 { (4, 3) } else { (2, 2) });
            g.finish_cleanup();
            assert_eq!(stats(&g, own), (2, 2));
            assert_eq!(stats(&g, foreign), (2, 2));
        }
    }
}

#[test]
fn spells_seize_opportunity_exiles_exact_cards_for_ordinary_paid_play_through_next_own_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Forest", Zone::Library);
        let bear = put(&mut g, p, "Bear Cub", Zone::Library);
        let source = put(&mut g, p, "Seize Opportunity", Zone::Hand);
        modal_cast(&mut g, source, 0, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Exile);
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
        let mut used = g.clone();
        let hand_land = put(&mut used, p, "Forest", Zone::Hand);
        act(&mut used, Action::PlayLand { card: hand_land });
        reject(&mut used, Action::PlayLand { card: land });
        let mut stale = g.clone();
        stale.move_object(bear, Zone::Hand);
        stale.move_object(bear, Zone::Exile);
        reject(&mut stale, Action::AnnounceExile { card: bear });
        g.finish_cleanup();
        g.finish_cleanup();
        assert_eq!(g.state.active, p);
        g.state.step = Step::Main1;
        g.state.priority = p;
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceExile { card: bear });
        let mut expired = g.clone();
        expired.finish_cleanup();
        expired.state.active = p;
        expired.state.priority = p;
        expired.state.step = Step::Main1;
        expired.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        reject(&mut expired, Action::AnnounceExile { card: bear });
        act(&mut g, Action::PlayLand { card: land });
        act(&mut g, Action::TapMana { card: land });
        g.state.players[p.index()].mana[5] = 1;
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceExile { card: bear }),
            "paid exact exile route: zone {:?}, actor {:?}, step {:?}, mana {:?}",
            g.object_view(bear).zone,
            g.actor(),
            g.state.step,
            g.state.players[p.index()].mana
        );
        act(&mut g, Action::AnnounceExile { card: bear });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn spells_worthy_cost_requires_own_creature_and_black_mana_and_is_sorcery_speed() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let artifact = put(&mut g, p, "Short Bow", Zone::Battlefield);
        let spell = put(&mut g, p, "Worthy Cost", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: spell });
        let material = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let mut upkeep = g.clone();
        upkeep.state.step = Step::Upkeep;
        reject(&mut upkeep, Action::AnnounceCast { card: spell });
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut g, Action::AnnounceCast { card: spell });
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
        act(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foreign),
            },
        );
        if matches!(g.state.pending, Pending::AdditionalCost { .. }) {
            let wrong = g.object_ref(foreign);
            reject(&mut g, Action::ChooseAdditionalCost { object: wrong });
        }
        pay_sacrifice(&mut g, material);
        settle(&mut g);
        assert_eq!(g.object_view(material).zone, Zone::Graveyard);
        assert_eq!(g.object_view(foreign).zone, Zone::Exile);
        assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn spells_worthy_cost_exiles_creatures_or_planeswalkers_and_paid_sacrifice_survives_countering() {
    for p in [Player::P0, Player::P1] {
        for route in 0..4 {
            let mut g = scenario(p);
            let material = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let target = if route == 0 {
                let id = put(
                    &mut g,
                    p.other(),
                    "Ajani, Caller of the Pride",
                    Zone::Battlefield,
                );
                g.set_counters(id, Counters::of(CounterKind::Loyalty, 5));
                id
            } else if route == 3 {
                material
            } else {
                put(&mut g, p.other(), "Bear Cub", Zone::Battlefield)
            };
            let spell = put(&mut g, p, "Worthy Cost", Zone::Hand);
            act(&mut g, Action::AnnounceCast { card: spell });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            pay_sacrifice(&mut g, material);
            assert_eq!(g.object_view(material).zone, Zone::Graveyard);
            if route == 1 {
                act(&mut g, Action::Pass);
                let cancel = put(&mut g, p.other(), "Cancel", Zone::Hand);
                cast(&mut g, cancel, Some(Target::Spell(spell)));
            }
            if route == 2 {
                g.move_object(target, Zone::Hand);
                g.move_object(target, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(target).zone,
                match route {
                    0 => Zone::Exile,
                    3 => Zone::Graveyard,
                    _ => Zone::Battlefield,
                }
            );
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        }
    }
}

fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        act(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn cost_clause(g: &mut Game, wanted: CastCostClause) {
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
        .position(|r| r.clause == wanted && !r.paid)
        .unwrap();
    act(
        g,
        Action::ChooseCastCost {
            clause: clause as u16,
        },
    );
}
fn granted_harmonize(g: &mut Game, spell: CardId, helper: Option<CardId>) {
    act(
        g,
        Action::AnnounceGrantedHarmonize {
            card: spell,
            face: CastFace::Normal,
        },
    );
    let object = helper.map(|id| g.object_ref(id));
    act(g, Action::ChooseHarmonizer { object });
    pay(g);
    if let Some(object) = object {
        cost_clause(g, CastCostClause::Harmonize(object));
    }
}
fn begin_delve(g: &mut Game) {
    act(g, Action::BeginDelve);
    cost_clause(g, CastCostClause::Mana);
}
fn delver(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    act(g, Action::ChooseDelver { object });
}
fn omen(g: &mut Game, id: CardId, x: Option<u32>, target: Option<Target>) {
    act(g, Action::AnnounceOmen { card: id });
    if let Some(value) = x {
        act(g, Action::ChooseX { value });
    }
    if let Some(target) = target {
        act(g, Action::ChooseTarget { target });
    }
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}

#[test]
fn cast_faces_songcrafter_has_flash_exact_graveyard_targets_and_taps_itself_for_printed_harmonize()
{
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let wanted = put(&mut g, p, "Quick Study", Zone::Graveyard);
        let foreign = put(&mut g, p.other(), "Quick Study", Zone::Graveyard);
        let dragon = put(&mut g, p, "Sagu Wildling // Roost Seek", Zone::Graveyard);
        let mage = put(&mut g, p, "Songcrafter Mage", Zone::Hand);
        g.state.active = p.other();
        g.state.step = Step::Upkeep;
        g.state.players[p.index()].mana = [0, 1, 0, 1, 1, 0];
        cast(&mut g, mage, None);
        reach(&mut g, |g| {
            g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(wanted),
            })
        });
        assert_eq!(stats(&g, mage), (3, 2));
        assert!(g.keywords(mage).contains(Keywords::FLASH));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(foreign),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(dragon),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(wanted),
            },
        );
        settle(&mut g);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        reject(
            &mut g,
            Action::AnnounceGrantedHarmonize {
                card: wanted,
                face: CastFace::Normal,
            },
        );
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        granted_harmonize(&mut g, wanted, Some(mage));
        settle(&mut g);
        assert!(g.object_view(mage).tapped);
        assert_eq!(g.object_view(wanted).zone, Zone::Exile);
        assert_eq!(g.hand_count(p), 2);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(dragon).zone, Zone::Graveyard);
    }
}

#[test]
fn cast_faces_songcrafter_grant_is_source_independent_exact_expires_and_exiles_a_countered_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p, "Quick Study", Zone::Graveyard);
        let other = put(&mut g, p, "Quick Study", Zone::Graveyard);
        let mage = put(&mut g, p, "Songcrafter Mage", Zone::Hand);
        cast(&mut g, mage, None);
        trigger_target(&mut g, Target::Card(target));
        settle(&mut g);
        g.move_object(mage, Zone::Graveyard);
        let route = Action::AnnounceGrantedHarmonize {
            card: target,
            face: CastFace::Normal,
        };
        assert!(g.legal_actions().contains(&route));
        reject(
            &mut g,
            Action::AnnounceGrantedHarmonize {
                card: other,
                face: CastFace::Normal,
            },
        );
        let mut stale = g.clone();
        stale.move_object(target, Zone::Exile);
        stale.move_object(target, Zone::Graveyard);
        reject(&mut stale, route.clone());
        let mut expired = g.clone();
        expired.finish_cleanup();
        expired.state.active = p;
        expired.state.priority = p;
        expired.state.step = Step::Main1;
        expired.state.players[p.index()].mana = [8; 6];
        reject(&mut expired, route);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
        granted_harmonize(&mut g, target, None);
        act(&mut g, Action::Pass);
        let cancel = put(&mut g, p.other(), "Cancel", Zone::Hand);
        cast(&mut g, cancel, Some(Target::Spell(target)));
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Exile);
        assert_eq!(g.object_view(other).zone, Zone::Graveyard);
        assert_eq!(g.hand_count(p), 0);
    }
}

#[test]
fn cast_faces_teval_does_not_grant_delve_or_cast_loss_from_hand_and_has_six_power_flying_lifelink()
{
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Forest", Zone::Graveyard);
        put(&mut g, p, "Forest", Zone::Graveyard);
        let teval = put(&mut g, p, "Teval, Arbiter of Virtue", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 1, 0, 1, 0];
        reject(&mut g, Action::AnnounceCast { card: teval });
        g.state.players[p.index()].mana = [0, 1, 1, 0, 1, 2];
        cast(&mut g, teval, None);
        settle(&mut g);
        assert_eq!(stats(&g, teval), (6, 6));
        assert!(
            g.keywords(teval)
                .contains(Keywords::FLYING.union(Keywords::LIFELINK))
        );
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.finish_cleanup();
        g.finish_cleanup();
        assert_eq!(g.state.active, p);
        assert!(!g.object_view(teval).summoning_sick);
        attack_one(&mut g, teval);
        reach(&mut g, |g| matches!(g.state.pending, Pending::Blockers));
        act(&mut g, Action::FinishBlockers);
        reach(&mut g, |g| g.state.step == Step::EndCombat);
        assert_eq!(g.state.players[p.index()].life, 26);
        assert_eq!(g.state.players[p.other().index()].life, 14);
    }
}

#[test]
fn cast_faces_teval_delve_pays_only_generic_and_cast_life_loss_survives_countering() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Teval, Arbiter of Virtue", Zone::Battlefield);
        let fuel: Vec<_> = (0..3)
            .map(|_| put(&mut g, p, "Forest", Zone::Graveyard))
            .collect();
        let foreign = put(&mut g, p.other(), "Forest", Zone::Graveyard);
        let spell = put(&mut g, p, "Quick Study", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        reject(&mut g, Action::AnnounceCast { card: spell });
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        act(&mut g, Action::AnnounceCast { card: spell });
        begin_delve(&mut g);
        let object = g.object_ref(foreign);
        reject(&mut g, Action::ChooseDelver { object });
        delver(&mut g, fuel[0]);
        delver(&mut g, fuel[1]);
        pay(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        act(&mut g, Action::Pass);
        let cancel = put(&mut g, p.other(), "Cancel", Zone::Hand);
        cast(&mut g, cancel, Some(Target::Spell(spell)));
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 17);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.object_view(fuel[0]).zone, Zone::Exile);
        assert_eq!(g.object_view(fuel[1]).zone, Zone::Exile);
        assert_eq!(g.object_view(fuel[2]).zone, Zone::Graveyard);
        assert_eq!(g.object_view(foreign).zone, Zone::Graveyard);
        assert_eq!(g.hand_count(p), 0);
    }
}

#[test]
fn cast_faces_teval_uses_selected_omen_mana_value_instead_of_the_printed_dragon_face() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Teval, Arbiter of Virtue", Zone::Battlefield);
        let card = put(&mut g, p, "Scavenger Regent // Exude Toxin", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 0];
        omen(&mut g, card, Some(0), None);
        let spell = g
            .state
            .stack
            .iter()
            .find(|s| s.kind == StackKind::Spell && s.source.is_some_and(|v| v.id == card))
            .unwrap();
        assert_eq!(spell.mana_value(), 2);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.object_view(card).zone, Zone::Library);
        assert_eq!(g.object_mana_value(card), 4);
        assert!(!g.can_play_adventure(p, card));
    }
}

#[test]
fn cast_faces_spectral_denial_uses_current_owned_four_power_threshold_but_counter_payment_is_chosen_x()
 {
    for p in [Player::P0, Player::P1] {
        for pay_tax in [false, true] {
            for x in [0, 2] {
                let mut g = scenario(p.other());
                let target = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
                cast(&mut g, target, None);
                act(&mut g, Action::Pass);
                let qualifying = put(&mut g, p, "Bear Cub", Zone::Battlefield);
                g.state.objects[qualifying as usize].growth.power = 2;
                put(&mut g, p, "Hill Giant", Zone::Battlefield);
                put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
                let mana_land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
                let denial = put(&mut g, p, "Spectral Denial", Zone::Hand);
                g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
                act(&mut g, Action::AnnounceCast { card: denial });
                reject(&mut g, Action::ChooseX { value: 3 });
                act(&mut g, Action::ChooseX { value: x });
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Spell(target),
                    },
                );
                pay(&mut g);
                assert_eq!(
                    g.state.players[p.index()].mana,
                    [0, 0, 0, 0, 0, u32::from(x == 0)]
                );
                g.state.players[p.other().index()].mana = [0, 0, 0, 0, 0, x];
                reach(&mut g, |g| {
                    matches!(g.state.pending, Pending::ResolutionMana { .. })
                });
                assert_eq!(g.actor(), Some(p.other()));
                if x == 0 {
                    reject(&mut g, Action::TapMana { card: mana_land });
                }
                if pay_tax {
                    act(
                        &mut g,
                        Action::PayResolutionMana {
                            payment: [0, 0, 0, 0, 0, x],
                        },
                    );
                } else {
                    act(&mut g, Action::DeclineResolutionMana);
                }
                settle(&mut g);
                assert_eq!(
                    g.object_view(target).zone,
                    if pay_tax {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    }
                );
            }
        }
    }
}

#[test]
fn cast_faces_spectral_denial_and_teval_use_x_mana_value_even_when_delve_pays_the_reduced_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
        cast(&mut g, target, None);
        act(&mut g, Action::Pass);
        put(&mut g, p, "Teval, Arbiter of Virtue", Zone::Battlefield);
        let a = put(&mut g, p, "Forest", Zone::Graveyard);
        let b = put(&mut g, p, "Forest", Zone::Graveyard);
        let denial = put(&mut g, p, "Spectral Denial", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        act(&mut g, Action::AnnounceCast { card: denial });
        act(&mut g, Action::ChooseX { value: 3 });
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Spell(target),
            },
        );
        begin_delve(&mut g);
        delver(&mut g, a);
        delver(&mut g, b);
        pay(&mut g);
        reach(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionMana { .. })
        });
        assert_eq!(g.state.players[p.index()].life, 16);
        act(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn cast_faces_scavenger_regent_ward_is_a_private_discard_cost_or_counters_the_targeting_spell() {
    for p in [Player::P0, Player::P1] {
        for discard in [false, true] {
            let mut g = scenario(p);
            let regent = put(&mut g, p, "Scavenger Regent // Exude Toxin", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
            cast(&mut g, regent, None);
            settle(&mut g);
            assert_eq!(stats(&g, regent), (4, 4));
            assert!(g.keywords(regent).contains(Keywords::FLYING));
            act(&mut g, Action::Pass);
            let material = put(&mut g, p.other(), "Forest", Zone::Hand);
            let mana_land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
            let shock = put(&mut g, p.other(), "Shock", Zone::Hand);
            cast(&mut g, shock, Some(Target::Card(regent)));
            reach(&mut g, |g| {
                g.legal_actions()
                    .contains(&Action::Discard { card: material })
            });
            assert_eq!(g.actor(), Some(p.other()));
            assert!(!g.observe(p).objects.iter().any(|o| o.id == material));
            reject(&mut g, Action::PayResolutionMana { payment: [0; 6] });
            reject(&mut g, Action::TapMana { card: mana_land });
            if discard {
                act(&mut g, Action::Discard { card: material });
            } else {
                act(&mut g, Action::DeclineResolutionMana);
            }
            settle(&mut g);
            assert_eq!(g.object_view(regent).damage, if discard { 2 } else { 0 });
            assert_eq!(
                g.object_view(material).zone,
                if discard { Zone::Graveyard } else { Zone::Hand }
            );
            assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn cast_faces_sagu_and_twinmaw_normal_creatures_pay_the_front_cost_and_gain_the_printed_life() {
    for p in [Player::P0, Player::P1] {
        for (name, color, generic, body, gain) in [
            ("Sagu Wildling // Roost Seek", 4, 4, (3, 3), 3),
            ("Twinmaw Stormbrood // Charring Bite", 0, 5, (5, 4), 5),
        ] {
            let mut g = scenario(p);
            let card = put(&mut g, p, name, Zone::Hand);
            let mut mana = [0; 6];
            mana[color] = 1;
            mana[5] = generic;
            g.state.players[p.index()].mana = mana;
            cast(&mut g, card, None);
            settle(&mut g);
            assert_eq!(stats(&g, card), body);
            assert!(g.keywords(card).contains(Keywords::FLYING));
            assert_eq!(g.state.players[p.index()].life, 20 + gain);
            assert_eq!(g.state.players[p.other().index()].life, 20);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(card).zone, Zone::Battlefield);
        }
    }
}

#[test]
fn cast_faces_stormshriek_front_has_haste_and_can_pump_while_tapped_until_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let feral = put(&mut g, p, "Stormshriek Feral // Flush Out", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 4];
        cast(&mut g, feral, None);
        settle(&mut g);
        assert_eq!(stats(&g, feral), (3, 3));
        assert!(
            g.keywords(feral)
                .contains(Keywords::FLYING.union(Keywords::HASTE))
        );
        attack_one(&mut g, feral);
        assert!(g.object_view(feral).tapped);
        g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 2];
        for _ in 0..2 {
            act(
                &mut g,
                Action::AnnounceActivate {
                    card: feral,
                    ability: 0,
                    counter_payment: Counters::default(),
                },
            );
            pay(&mut g);
            settle(&mut g);
        }
        assert_eq!(stats(&g, feral), (5, 3));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        reach(&mut g, |g| matches!(g.state.pending, Pending::Blockers));
        act(&mut g, Action::FinishBlockers);
        reach(&mut g, |g| g.state.step == Step::EndCombat);
        assert_eq!(g.state.players[p.other().index()].life, 15);
        g.finish_cleanup();
        assert_eq!(stats(&g, feral), (3, 3));
    }
}

#[test]
fn cast_faces_whirlwing_front_flashes_and_grants_only_sorcery_or_dragon_timing_while_present() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.state.active = p.other();
        g.state.step = Step::Upkeep;
        let source = put(
            &mut g,
            p,
            "Whirlwing Stormbrood // Dynamic Soar",
            Zone::Hand,
        );
        cast(&mut g, source, None);
        settle(&mut g);
        priority(&mut g, p);
        assert_eq!(stats(&g, source), (4, 3));
        assert!(
            g.keywords(source)
                .contains(Keywords::FLYING.union(Keywords::FLASH))
        );
        let sorcery = put(&mut g, p, "Unending Whisper", Zone::Hand);
        let dragon = put(&mut g, p, "Sagu Wildling // Roost Seek", Zone::Hand);
        let ordinary = put(&mut g, p, "Bear Cub", Zone::Hand);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: sorcery })
        );
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: dragon })
        );
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceOmen { card: dragon })
        );
        reject(&mut g, Action::AnnounceCast { card: ordinary });
        let mut cast_dragon = g.clone();
        cast(&mut cast_dragon, dragon, None);
        settle(&mut cast_dragon);
        assert_eq!(cast_dragon.object_view(dragon).zone, Zone::Battlefield);
        g.move_object(source, Zone::Graveyard);
        reject(&mut g, Action::AnnounceCast { card: sorcery });
        reject(&mut g, Action::AnnounceCast { card: dragon });
        reject(&mut g, Action::AnnounceOmen { card: dragon });
    }
}

const OMENS: &[&str] = &[
    "Sagu Wildling // Roost Seek",
    "Scavenger Regent // Exude Toxin",
    "Stormshriek Feral // Flush Out",
    "Twinmaw Stormbrood // Charring Bite",
    "Whirlwing Stormbrood // Dynamic Soar",
];
#[test]
fn cast_faces_each_omen_is_a_sorcery_even_if_its_front_creature_has_flash() {
    for p in [Player::P0, Player::P1] {
        for &name in OMENS {
            let mut g = scenario(p);
            put(&mut g, p, "Bear Cub", Zone::Battlefield);
            put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let card = put(&mut g, p, name, Zone::Hand);
            g.state.step = Step::Upkeep;
            reject(&mut g, Action::AnnounceOmen { card });
            g.state.step = Step::Main1;
            g.state.active = p.other();
            reject(&mut g, Action::AnnounceOmen { card });
        }
    }
}

#[test]
fn cast_faces_roost_seek_searches_only_basic_lands_privately_reveals_the_pick_and_shuffles_the_dragon()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let chosen = put(&mut g, p, "Island", Zone::Library);
        let nonbasic = put(&mut g, p, "Sandsteppe Citadel", Zone::Library);
        let card = put(&mut g, p, "Sagu Wildling // Roost Seek", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        omen(&mut g, card, None, None);
        reach(&mut g, |g| {
            g.legal_actions()
                .contains(&Action::ChooseSearch { card: chosen })
        });
        assert!(g.observe(p.other()).search_choice.is_none());
        reject(&mut g, Action::ChooseSearch { card: nonbasic });
        act(&mut g, Action::ChooseSearch { card: chosen });
        act(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.object_view(chosen).zone, Zone::Hand);
        assert!(
            g.observe(p.other())
                .known_hand
                .contains(&g.object_ref(chosen))
        );
        assert!(
            !g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == nonbasic)
        );
        assert_eq!(g.object_view(card).zone, Zone::Library);
        assert!(!g.can_play_adventure(p, card));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn cast_faces_exude_toxin_uses_x_on_the_omen_face_and_changes_only_current_non_dragons_until_cleanup()
 {
    for p in [Player::P0, Player::P1] {
        for x in [0, 2] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let dragon = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
            let card = put(&mut g, p, "Scavenger Regent // Exude Toxin", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 2, 0, 0, x];
            omen(&mut g, card, Some(x), None);
            let spell = g
                .state
                .stack
                .iter()
                .find(|s| s.kind == StackKind::Spell && s.source.is_some_and(|v| v.id == card))
                .unwrap();
            assert_eq!(spell.mana_value(), u64::from(x) + 2);
            settle(&mut g);
            assert_eq!(stats(&g, own), (3 - i128::from(x), 3 - i128::from(x)));
            assert_eq!(stats(&g, enemy), (3 - i128::from(x), 3 - i128::from(x)));
            assert_eq!(stats(&g, dragon), (5, 5));
            assert_eq!(g.object_view(card).zone, Zone::Library);
            assert_eq!(g.object_mana_value(card), 4);
            let late = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            assert_eq!(stats(&g, late), (2, 2));
            g.finish_cleanup();
            assert_eq!(stats(&g, own), (3, 3));
            assert_eq!(stats(&g, enemy), (3, 3));
        }
    }
}

#[test]
fn cast_faces_charring_bite_uses_red_cost_targets_only_nonfliers_and_fizzles_to_graveyard() {
    for p in [Player::P0, Player::P1] {
        for lose_target in [false, true] {
            let mut g = scenario(p);
            let target = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            g.state.objects[target as usize].growth.toughness = 3;
            let flyer = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
            let card = put(&mut g, p, "Twinmaw Stormbrood // Charring Bite", Zone::Hand);
            g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
            reject(&mut g, Action::AnnounceOmen { card });
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
            act(&mut g, Action::AnnounceOmen { card });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(flyer),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(target),
                },
            );
            pay(&mut g);
            if lose_target {
                g.state.objects[target as usize].gained_keywords = Keywords::FLYING;
            }
            settle(&mut g);
            assert_eq!(
                g.object_view(target).damage,
                if lose_target { 0 } else { 5 }
            );
            assert_eq!(
                g.object_view(card).zone,
                if lose_target {
                    Zone::Graveyard
                } else {
                    Zone::Library
                }
            );
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(!g.can_play_adventure(p, card));
        }
    }
}

#[test]
fn cast_faces_dynamic_soar_places_three_persistent_counters_only_on_own_exact_creature() {
    for p in [Player::P0, Player::P1] {
        for blink in [false, true] {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let card = put(
                &mut g,
                p,
                "Whirlwing Stormbrood // Dynamic Soar",
                Zone::Hand,
            );
            act(&mut g, Action::AnnounceOmen { card });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(foreign),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            pay(&mut g);
            if blink {
                g.move_object(own, Zone::Hand);
                g.move_object(own, Zone::Battlefield);
            }
            settle(&mut g);
            assert_eq!(
                g.counter_count(own, CounterKind::PlusOne),
                if blink { 0 } else { 3 }
            );
            assert_eq!(
                g.object_view(card).zone,
                if blink {
                    Zone::Graveyard
                } else {
                    Zone::Library
                }
            );
            g.finish_cleanup();
            assert_eq!(stats(&g, own), if blink { (2, 2) } else { (5, 5) });
        }
    }
}

#[test]
fn cast_faces_flush_out_discards_before_drawing_two_and_draws_nothing_with_an_empty_hand() {
    for p in [Player::P0, Player::P1] {
        for has_card in [false, true] {
            let mut g = scenario(p);
            let top = put(&mut g, p, "Bear Cub", Zone::Library);
            let next = put(&mut g, p, "Island", Zone::Library);
            let material = has_card.then(|| put(&mut g, p, "Forest", Zone::Hand));
            let card = put(&mut g, p, "Stormshriek Feral // Flush Out", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 1];
            omen(&mut g, card, None, None);
            if let Some(material) = material {
                reach(&mut g, |g| {
                    g.legal_actions()
                        .contains(&Action::Discard { card: material })
                });
                assert_eq!(g.object_view(top).zone, Zone::Library);
                assert_eq!(g.object_view(next).zone, Zone::Library);
                assert!(g.observe(p.other()).effect_choice.is_none());
                act(&mut g, Action::Discard { card: material });
            }
            settle(&mut g);
            assert_eq!(g.hand_count(p), if has_card { 2 } else { 0 });
            assert_eq!(
                g.object_view(top).zone,
                if has_card { Zone::Hand } else { Zone::Library }
            );
            assert_eq!(
                g.object_view(next).zone,
                if has_card { Zone::Hand } else { Zone::Library }
            );
            assert_eq!(g.object_view(card).zone, Zone::Library);
            assert!(!g.can_play_adventure(p, card));
        }
    }
}

#[test]
fn cast_faces_all_five_countered_omens_go_to_graveyard_instead_of_shuffling_or_exiling() {
    for p in [Player::P0, Player::P1] {
        for &name in OMENS {
            let mut g = scenario(p);
            let own = put(&mut g, p, "Hill Giant", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
            let card = put(&mut g, p, name, Zone::Hand);
            omen(
                &mut g,
                card,
                (name == OMENS[1]).then_some(0),
                if name == OMENS[3] {
                    Some(Target::Card(enemy))
                } else if name == OMENS[4] {
                    Some(Target::Card(own))
                } else {
                    None
                },
            );
            act(&mut g, Action::Pass);
            let cancel = put(&mut g, p.other(), "Cancel", Zone::Hand);
            cast(&mut g, cancel, Some(Target::Spell(card)));
            settle(&mut g);
            assert_eq!(g.object_view(card).zone, Zone::Graveyard, "{name}");
            assert!(!g.can_play_adventure(p, card));
            assert_eq!(g.counter_count(own, CounterKind::PlusOne), 0);
            assert_eq!(g.object_view(enemy).damage, 0);
        }
    }
}

#[test]
fn cast_faces_actual_teval_delve_selection_is_owned_raw_index_restorable_and_does_not_leak_draws() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Teval, Arbiter of Virtue", Zone::Battlefield);
        let fuel: Vec<_> = (0..3)
            .map(|_| put(&mut g, p, "Forest", Zone::Graveyard))
            .collect();
        let hidden = put(&mut g, p, "Cancel", Zone::Library);
        let spell = put(&mut g, p, "Quick Study", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        act(&mut g, Action::AnnounceCast { card: spell });
        begin_delve(&mut g);
        delver(&mut g, fuel[0]);
        let action = Action::ChooseDelver {
            object: g.object_ref(fuel[1]),
        };
        let revision = g.revision();
        let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        let mut sibling = g.clone();
        drop(g);
        raw.apply(p, action.clone()).unwrap();
        indexed.apply_index(p, revision, index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        delver(&mut sibling, fuel[2]);
        for child in [&mut raw, &mut indexed, &mut sibling] {
            pay(child);
            settle(child);
            assert_eq!(child.state.players[p.index()].life, 17);
            assert_eq!(child.hand_count(p), 2);
            assert!(
                !child
                    .observe(p.other())
                    .objects
                    .iter()
                    .any(|o| o.id == hidden)
            );
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(sibling.object_view(fuel[1]).zone, Zone::Graveyard);
        assert_eq!(raw.object_view(fuel[1]).zone, Zone::Exile);
        indexed.restore(&saved).unwrap();
        assert_eq!(
            indexed.apply_index(p, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut indexed, action);
        pay(&mut indexed);
        settle(&mut indexed);
        assert_eq!(raw.snapshot(), indexed.snapshot());
    }
}

#[test]
fn cast_faces_songcrafter_grants_sorcery_printed_cost_but_flash_does_not_transfer_to_it() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.state.active = p.other();
        g.state.step = Step::Upkeep;
        let spell = put(&mut g, p, "Unending Whisper", Zone::Graveyard);
        let mage = put(&mut g, p, "Songcrafter Mage", Zone::Hand);
        cast(&mut g, mage, None);
        trigger_target(&mut g, Target::Card(spell));
        settle(&mut g);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        reject(
            &mut g,
            Action::AnnounceGrantedHarmonize {
                card: spell,
                face: CastFace::Normal,
            },
        );
        let provider = put(
            &mut g,
            p,
            "Whirlwing Stormbrood // Dynamic Soar",
            Zone::Battlefield,
        );
        reject(&mut g, Action::AnnounceHarmonize { card: spell });
        granted_harmonize(&mut g, spell, None);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert_eq!(g.hand_count(p), 1);
        assert!(!g.object_view(mage).tapped);
        assert!(!g.object_view(provider).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}
