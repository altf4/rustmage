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
