//! Printed-outcome proofs for the final six inherited HOB identities.
use super::*;
const FIXTURES: &[&str] = &[
    "Giant's Boulder",
    "Hobbit Hole",
    "Gundabad Opportunist",
    "Stony-Voiced Goblins",
    "The Sackville-Bagginses",
    "Troop of Ponies",
    "Forest",
    "Island",
    "Shock",
    "Bear Cub",
    "Goldvein Pick",
    "Pacifism",
    "@token/hob-wolf",
    "@token/b60f6cd3b44b22bd289fe4e2",
];
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(
        FIXTURES.contains(&name),
        "fixture missing from explicit preflight: {name}"
    );
    put(g, p, name, zone)
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.resolving.is_none()
        && g.state.ordering_triggers.is_empty()
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..40 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("remaining proof resolution budget: {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, quiet);
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana) -> CardId {
    let id = fixture(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn activation(id: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card: id,
        ability,
        counter_payment: Counters::default(),
    }
}
fn mana_activation(id: CardId, production: Mana) -> Action {
    Action::AnnounceMana {
        card: id,
        ability: 0,
        counter_payment: Counters::default(),
        production,
    }
}
fn search(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectSearch { .. })
    });
}
fn library(g: &mut Game, p: Player, names: &[&str]) -> Vec<CardId> {
    // Scenario's ordinary eight basic cards stay underneath the named suffix.
    names
        .iter()
        .map(|name| fixture(g, p, name, Zone::Library))
        .collect()
}
fn private(g: &Game, p: Player, ids: &[CardId]) {
    assert!(
        g.observe(p.other())
            .objects
            .iter()
            .all(|o| !ids.contains(&o.id))
    );
}
fn next_main(g: &mut Game) {
    let turn = g.state.turn;
    for _ in 0..80 {
        if g.state.turn != turn && g.state.step == Step::Main1 && quiet(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    panic!("next main phase budget");
}
fn treasure(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| {
            o.zone == Zone::Battlefield && o.controller == p && o.card.has_subtype("Treasure")
        })
        .map(|(id, _)| id as CardId)
        .collect()
}
#[test]
fn hob_inherited_remaining_boulder_paid_scry_two_private_top_bottom_and_order() {
    for p in [Player::P0, Player::P1] {
        for mask in 0..4 {
            let mut g = scenario(p);
            let ids = library(&mut g, p, &["Forest", "Island", "Shock"]);
            let boulder = paid(&mut g, p, "Giant's Boulder", [0, 0, 0, 0, 0, 1]);
            until(&mut g, |g| g.observe(p).library_choice.is_some());
            assert_eq!(g.object_view(boulder).kinds, 16);
            let looked: Vec<_> = g
                .observe(p)
                .library_choice
                .unwrap()
                .remaining
                .iter()
                .map(|o| o.id)
                .collect();
            assert_eq!(looked, vec![ids[2], ids[1]]);
            assert!(g.observe(p.other()).library_choice.is_none());
            private(&g, p, &ids);
            reject(
                &mut g,
                Action::ArrangeLibrary {
                    card: ids[0],
                    destination: LibraryDestination::Top,
                },
            );
            for (i, &id) in [ids[1], ids[2]].iter().enumerate() {
                act(
                    &mut g,
                    Action::ArrangeLibrary {
                        card: id,
                        destination: if mask & (1 << i) == 0 {
                            LibraryDestination::Top
                        } else {
                            LibraryDestination::Bottom
                        },
                    },
                );
            }
            finish(&mut g);
            let top: Vec<_> = [ids[1], ids[2]]
                .into_iter()
                .enumerate()
                .filter_map(|(i, id)| (mask & (1 << i) == 0).then_some(id))
                .collect();
            let bottom: Vec<_> = [ids[1], ids[2]]
                .into_iter()
                .enumerate()
                .filter_map(|(i, id)| (mask & (1 << i) != 0).then_some(id))
                .collect();
            let lib = &g.state.players[p.index()].library;
            assert_eq!(
                lib.iter()
                    .rev()
                    .take(top.len())
                    .copied()
                    .collect::<Vec<_>>(),
                top
            );
            assert_eq!(
                lib.iter()
                    .take(bottom.len())
                    .rev()
                    .copied()
                    .collect::<Vec<_>>(),
                bottom
            );
            assert_eq!(lib.len(), 11);
            private(&g, p, &ids);
        }
    }
}
fn boulder(g: &mut Game, p: Player) -> CardId {
    let id = paid(g, p, "Giant's Boulder", [0, 0, 0, 0, 0, 1]);
    until(g, |g| g.observe(p).library_choice.is_some());
    for _ in 0..2 {
        let card = g.observe(p).library_choice.unwrap().remaining[0].id;
        act(
            g,
            Action::ArrangeLibrary {
                card,
                destination: LibraryDestination::Top,
            },
        );
    }
    finish(g);
    id
}
#[test]
fn hob_inherited_remaining_boulder_one_and_tap_five_colors_immediate_mana_not_colorless() {
    for p in [Player::P0, Player::P1] {
        for color in 0..5 {
            let mut g = scenario(p);
            let id = boulder(&mut g, p);
            g.state.active = p.other();
            g.state.priority = p;
            g.state.players[p.index()].mana = [0; 6];
            let mut production = [0; 6];
            production[color] = 1;
            reject(&mut g, mana_activation(id, production));
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            reject(&mut g, mana_activation(id, [0, 0, 0, 0, 0, 1]));
            act(&mut g, mana_activation(id, production));
            pay(&mut g);
            assert_eq!(g.state.players[p.index()].mana, production);
            assert!(g.object_view(id).tapped);
            assert_eq!(g.object_view(id).zone, Zone::Battlefield);
            assert!(g.state.stack.is_empty());
            reject(&mut g, mana_activation(id, production));
        }
    }
}
#[test]
fn hob_inherited_remaining_boulder_seven_tap_sacrifice_any_permanent_stale_and_indestructible() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..7 {
            let mut g = scenario(p);
            let id = boulder(&mut g, p);
            let name = [
                "Bear Cub",
                "Forest",
                "Goldvein Pick",
                "Pacifism",
                "Bear Cub",
                "Bear Cub",
                "Bear Cub",
            ][mode];
            let victim = fixture(
                &mut g,
                if mode == 4 { p } else { p.other() },
                name,
                Zone::Battlefield,
            );
            if name == "Pacifism" {
                let anchor = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
                g.state.objects[victim as usize].attached_to = Some(g.object_ref(anchor));
            }
            let hand = fixture(&mut g, p.other(), "Forest", Zone::Hand);
            if mode == 6 {
                g.state.objects[victim as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
            }
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 6];
            reject(&mut g, activation(id, 1));
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 7];
            act(&mut g, activation(id, 1));
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(hand),
                },
            );
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
            if mode == 5 {
                g.move_object(victim, Zone::Hand);
                g.move_object(victim, Zone::Battlefield);
            }
            finish(&mut g);
            assert_eq!(
                g.object_view(victim).zone,
                if mode >= 5 {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
        }
    }
}
#[test]
fn hob_inherited_remaining_hole_land_play_tap_sacrifice_private_basic_search_shuffle_owner() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = scenario(p);
            let ids = library(&mut g, p, &["Forest", "Island", "Hobbit Hole", "Shock"]);
            let foreign = fixture(&mut g, p.other(), "Forest", Zone::Library);
            let hole = fixture(
                &mut g,
                if mode == 2 { p.other() } else { p },
                "Hobbit Hole",
                if mode == 2 {
                    Zone::Battlefield
                } else {
                    Zone::Hand
                },
            );
            if mode == 2 {
                g.record_control(g.object_ref(hole), p, None);
            } else {
                act(&mut g, Action::PlayLand { card: hole });
            }
            assert!(!g.object_view(hole).tapped);
            g.state.active = p.other();
            g.state.priority = p;
            g.state.players[p.index()].mana = [0; 6];
            act(&mut g, activation(hole, 0));
            pay(&mut g);
            assert_eq!(g.object_view(hole).zone, Zone::Graveyard);
            assert_eq!(
                g.object_view(hole).owner,
                if mode == 2 { p.other() } else { p }
            );
            search(&mut g);
            assert!(g.observe(p).search_choice.is_some());
            assert!(g.observe(p.other()).search_choice.is_none());
            private(&g, p, &ids);
            for card in [ids[2], ids[3], foreign] {
                reject(&mut g, Action::ChooseSearch { card });
            }
            let rng = g.state.rng;
            if mode != 0 {
                act(&mut g, Action::ChooseSearch { card: ids[1] });
            }
            act(&mut g, Action::FinishSearch);
            finish(&mut g);
            assert_ne!(g.state.rng, rng);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.object_view(ids[1]).zone,
                if mode == 0 {
                    Zone::Library
                } else {
                    Zone::Battlefield
                }
            );
            if mode != 0 {
                assert!(g.object_view(ids[1]).tapped);
                assert_eq!(g.object_view(ids[1]).controller, p);
            }
            assert_eq!(g.object_view(foreign).zone, Zone::Library);
        }
    }
}
#[test]
fn hob_inherited_remaining_hole_four_hand_discard_halflingcycling_reveal_and_fail_to_find() {
    for p in [Player::P0, Player::P1] {
        for find in [false, true] {
            let mut g = scenario(p);
            let ids = library(
                &mut g,
                p,
                &["The Sackville-Bagginses", "Bear Cub", "Forest"],
            );
            let hole = fixture(&mut g, p, "Hobbit Hole", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            reject(&mut g, activation(hole, 1));
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
            act(&mut g, activation(hole, 1));
            pay(&mut g);
            assert_eq!(g.object_view(hole).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            search(&mut g);
            private(&g, p, &ids);
            assert!(g.observe(p.other()).search_choice.is_none());
            for &card in &ids[1..] {
                reject(&mut g, Action::ChooseSearch { card });
            }
            let rng = g.state.rng;
            if find {
                act(&mut g, Action::ChooseSearch { card: ids[0] });
                private(&g, p, &ids);
            }
            act(&mut g, Action::FinishSearch);
            finish(&mut g);
            assert_ne!(g.state.rng, rng);
            assert_eq!(
                g.object_view(ids[0]).zone,
                if find { Zone::Hand } else { Zone::Library }
            );
            assert_eq!(g.hand_count(p), usize::from(find));
            assert_eq!(
                g.observe(p.other()).objects.iter().any(|o| o.id == ids[0]),
                find
            );
            private(&g, p, &ids[1..]);
            reject(&mut g, activation(hole, 1));
        }
    }
}
#[test]
fn hob_inherited_remaining_ponies_paid_tap_sacrifice_zero_one_two_reveal_split_owned_shuffle() {
    for p in [Player::P0, Player::P1] {
        for count in 0..3 {
            let mut g = scenario(p);
            let ids = library(&mut g, p, &["Forest", "Island", "Hobbit Hole", "Shock"]);
            let source = paid(&mut g, p, "Troop of Ponies", [0, 0, 0, 0, 0, 2]);
            finish(&mut g);
            assert_eq!(stats(&g, source), (2, 1));
            assert!(g.has_subtype(source, "Horse"));
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            reject(&mut g, activation(source, 0));
            g.state.objects[source as usize].summoning_sick = false;
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
            reject(&mut g, activation(source, 0));
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
            act(&mut g, activation(source, 0));
            pay(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            search(&mut g);
            private(&g, p, &ids);
            assert!(g.observe(p.other()).search_choice.is_none());
            for &card in &ids[2..] {
                reject(&mut g, Action::ChooseSearch { card });
            }
            for &card in ids.iter().take(count) {
                act(&mut g, Action::ChooseSearch { card });
            }
            private(&g, p, &ids);
            let rng = g.state.rng;
            act(&mut g, Action::FinishSearch);
            if count == 2 {
                let revealed = g.observe(p.other()).search_distribution.unwrap();
                assert_eq!(
                    revealed.candidates.iter().map(|o| o.id).collect::<Vec<_>>(),
                    ids[..2]
                );
                private(&g, p, &ids[2..]);
                contracts::branch_contract(g.clone());
                let saved = g.snapshot();
                let mut indexed = g.clone();
                let action = Action::ChooseSearchBattlefield { card: ids[1] };
                let index = indexed
                    .legal_actions()
                    .iter()
                    .position(|a| a == &action)
                    .unwrap();
                indexed.apply_index(p, indexed.revision(), index).unwrap();
                act(&mut g, action.clone());
                assert_eq!(g.snapshot(), indexed.snapshot());
                indexed.restore(&saved).unwrap();
                act(&mut indexed, action);
                assert_eq!(g.snapshot(), indexed.snapshot());
                drop(indexed);
            }
            finish(&mut g);
            assert_ne!(g.state.rng, rng);
            if count == 1 {
                assert_eq!(g.object_view(ids[0]).zone, Zone::Battlefield);
                assert!(g.object_view(ids[0]).tapped);
            }
            if count == 2 {
                assert_eq!(g.object_view(ids[1]).zone, Zone::Battlefield);
                assert!(g.object_view(ids[1]).tapped);
                assert_eq!(g.object_view(ids[0]).zone, Zone::Hand);
                assert!(
                    g.observe(p.other())
                        .objects
                        .iter()
                        .any(|o| o.id == ids[0] && o.zone == Zone::Hand)
                );
            }
            if count == 0 {
                assert_eq!(g.object_view(ids[0]).zone, Zone::Library);
            }
            assert_eq!(g.hand_count(p), usize::from(count == 2));
            reject(&mut g, activation(source, 0));
        }
    }
}
#[test]
fn hob_inherited_remaining_opportunist_paid_exile_public_land_and_normal_paid_spell_timing() {
    for p in [Player::P0, Player::P1] {
        for name in ["Forest", "Bear Cub", "Shock"] {
            let mut g = scenario(p);
            let top = fixture(&mut g, p, name, Zone::Library);
            let source = paid(&mut g, p, "Gundabad Opportunist", [0, 0, 0, 1, 0, 3]);
            finish(&mut g);
            assert_eq!(stats(&g, source), (4, 2));
            assert!(g.has_subtype(source, "Goblin") && g.has_subtype(source, "Rogue"));
            assert_eq!(g.object_view(top).zone, Zone::Exile);
            assert!(
                g.observe(p.other())
                    .objects
                    .iter()
                    .any(|o| o.id == top && o.zone == Zone::Exile)
            );
            assert!(g.can_play_exiled(p, top));
            assert!(!g.can_play_exiled(p.other(), top));
            g.move_object(source, Zone::Graveyard);
            if name == "Forest" {
                act(&mut g, Action::PlayLand { card: top });
                assert_eq!(g.object_view(top).zone, Zone::Battlefield);
                let other = fixture(&mut g, p, "Island", Zone::Hand);
                reject(&mut g, Action::PlayLand { card: other });
            } else {
                g.state.active = p.other();
                g.state.priority = p;
                g.state.players[p.index()].mana = [0; 6];
                reject(&mut g, Action::AnnounceExile { card: top });
                if name == "Bear Cub" {
                    g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
                    reject(&mut g, Action::AnnounceExile { card: top });
                    g.state.active = p;
                    g.state.priority = p;
                } else {
                    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
                }
                act(&mut g, Action::AnnounceExile { card: top });
                if name == "Shock" {
                    act(
                        &mut g,
                        Action::ChooseTarget {
                            target: Target::Player(p.other()),
                        },
                    );
                    if g.legal_actions().contains(&Action::FinishTargets) {
                        act(&mut g, Action::FinishTargets);
                    }
                }
                pay(&mut g);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                finish(&mut g);
                assert_eq!(
                    g.object_view(top).zone,
                    if name == "Shock" {
                        Zone::Graveyard
                    } else {
                        Zone::Battlefield
                    }
                );
                if name == "Shock" {
                    assert_eq!(g.state.players[p.other().index()].life, 18);
                } else {
                    assert_eq!(stats(&g, top), (2, 2));
                }
            }
            assert!(!g.can_play_exiled(p, top));
        }
    }
}
#[test]
fn hob_inherited_remaining_opportunist_exact_permission_through_next_turn_cleanup_and_empty_library()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let top = fixture(&mut g, p, "Bear Cub", Zone::Library);
        paid(&mut g, p, "Gundabad Opportunist", [0, 0, 0, 1, 0, 3]);
        finish(&mut g);
        let mut stale = g.clone();
        stale.move_object(top, Zone::Hand);
        stale.move_object(top, Zone::Exile);
        assert!(!stale.can_play_exiled(p, top));
        next_main(&mut g);
        assert_eq!(g.state.active, p.other());
        assert!(g.can_play_exiled(p, top));
        next_main(&mut g);
        assert_eq!(g.state.active, p);
        assert!(g.can_play_exiled(p, top));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceExile { card: top })
        );
        next_main(&mut g);
        assert!(!g.can_play_exiled(p, top));
        g.state.priority = p;
        reject(&mut g, Action::AnnounceExile { card: top });
        let mut g = scenario(p);
        g.state.players[p.index()].library.clear();
        paid(&mut g, p, "Gundabad Opportunist", [0, 0, 0, 1, 0, 3]);
        finish(&mut g);
        assert!(g.state.exile_permissions.is_empty());
    }
}
#[test]
fn hob_inherited_remaining_stony_paid_opponent_private_discard_choice_empty_and_source_departure() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = scenario(p);
            let own = fixture(&mut g, p, "Island", Zone::Hand);
            let hand: Vec<_> = if mode == 0 {
                Vec::new()
            } else {
                ["Forest", "Shock"]
                    .iter()
                    .map(|n| fixture(&mut g, p.other(), n, Zone::Hand))
                    .collect()
            };
            g.state.players[p.other().index()].hexproof_until_end_turn = true;
            let source = paid(&mut g, p, "Stony-Voiced Goblins", [0, 0, 1, 0, 0, 1]);
            if mode == 0 {
                finish(&mut g);
            } else {
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectDiscard { .. })
                });
                assert_eq!(g.actor(), Some(p.other()));
                assert_eq!(stats(&g, source), (1, 1));
                assert!(g.has_subtype(source, "Goblin") && g.has_subtype(source, "Bard"));
                assert!(g.observe(p).objects.iter().all(|o| !hand.contains(&o.id)));
                reject(&mut g, Action::Discard { card: own });
                if mode == 2 {
                    g.move_object(source, Zone::Graveyard);
                }
                act(&mut g, Action::Discard { card: hand[1] });
                finish(&mut g);
                assert_eq!(g.object_view(hand[1]).zone, Zone::Graveyard);
                assert_eq!(g.object_view(hand[0]).zone, Zone::Hand);
                assert!(g.observe(p).objects.iter().any(|o| o.id == hand[1]));
                assert!(g.observe(p).objects.iter().all(|o| o.id != hand[0]));
            }
            assert_eq!(g.object_view(own).zone, Zone::Hand);
            assert_eq!(g.hand_count(p), 1);
            assert_eq!(g.hand_count(p.other()), usize::from(mode != 0));
        }
    }
}
#[test]
fn hob_inherited_remaining_sackville_optional_other_creature_artifact_token_owner_and_draw_treasure()
 {
    for p in [Player::P0, Player::P1] {
        for mode in 0..7 {
            let mut g = scenario(p);
            let name = match mode {
                2 => "Goldvein Pick",
                3 | 5 => "@token/hob-wolf",
                4 => "@token/b60f6cd3b44b22bd289fe4e2",
                _ => "Bear Cub",
            };
            let offering = fixture(
                &mut g,
                if mode == 5 { p.other() } else { p },
                name,
                Zone::Battlefield,
            );
            if mode == 5 {
                g.record_control(g.object_ref(offering), p, None);
            }
            let enemy = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let land = fixture(&mut g, p, "Forest", Zone::Battlefield);
            let enchantment = fixture(&mut g, p, "Pacifism", Zone::Battlefield);
            g.state.objects[enchantment as usize].attached_to = Some(g.object_ref(enemy));
            let drawn = fixture(&mut g, p, "Island", Zone::Library);
            let source = paid(&mut g, p, "The Sackville-Bagginses", [0, 0, 1, 0, 0, 1]);
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectChoice { .. })
            });
            assert_eq!(stats(&g, source), (2, 2));
            assert!(g.object_view(source).is_legendary());
            assert!(g.has_subtype(source, "Halfling") && g.has_subtype(source, "Citizen"));
            for id in [source, enemy, land, enchantment] {
                let object = g.object_ref(id);
                reject(&mut g, Action::Sacrifice { object });
            }
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            if mode == 0 {
                act(&mut g, Action::DeclineEffectPayment);
            } else {
                if mode == 1 {
                    contracts::branch_contract(g.clone());
                }
                if mode == 2 {
                    g.move_object(source, Zone::Graveyard);
                }
                let object = g.object_ref(offering);
                act(&mut g, Action::Sacrifice { object });
            }
            // Sacrificed-token trigger is targeted; choose only the opponent.
            for _ in 0..32 {
                if quiet(&g) {
                    break;
                }
                if g.legal_actions().contains(&Action::ChooseTarget {
                    target: Target::Player(p.other()),
                }) {
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
                    if g.legal_actions().contains(&Action::FinishTargets) {
                        act(&mut g, Action::FinishTargets);
                    }
                } else {
                    choose(&mut g, |a| {
                        matches!(a, Action::Pass | Action::PutTrigger { .. })
                    });
                }
            }
            assert!(quiet(&g));
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                u64::from(mode != 0)
            );
            assert_eq!(
                g.object_view(drawn).zone,
                if mode == 0 { Zone::Library } else { Zone::Hand }
            );
            private(&g, p, &[drawn]);
            let token_sac = matches!(mode, 3 | 4 | 5);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if token_sac { 19 } else { 20 }
            );
            assert_eq!(treasure(&g, p).len(), if mode == 0 { 0 } else { 1 });
            if mode != 0 {
                let t = treasure(&g, p)[0];
                assert!(g.object_view(t).is_token());
                assert_eq!(g.object_view(t).kinds, 16);
                assert_eq!(g.object_view(t).colors, 0);
                assert!(!g.object_view(t).tapped);
            }
            if mode == 6 {
                // A new Treasure's genuine tap/sacrifice mana payment fires once.
                let t = treasure(&g, p)[0];
                act(&mut g, mana_activation(t, [0, 0, 0, 0, 1, 0]));
                pay(&mut g);
                until(&mut g, |g| {
                    g.legal_actions().contains(&Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    })
                });
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
                if g.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                }
                finish(&mut g);
                assert_eq!(g.state.players[p.other().index()].life, 19);
                assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);
            }
        }
    }
}
#[test]
fn hob_inherited_remaining_sackville_no_offering_opponent_sacrifice_and_nonsacrifice_death_excluded()
 {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = paid(&mut g, p, "The Sackville-Bagginses", [0, 0, 1, 0, 0, 1]);
        finish(&mut g);
        assert!(treasure(&g, p).is_empty());
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        let token = fixture(
            &mut g,
            p.other(),
            "@token/b60f6cd3b44b22bd289fe4e2",
            Zone::Battlefield,
        );
        g.state.priority = p.other();
        g.state.players[p.other().index()].mana = [0; 6];
        act(&mut g, mana_activation(token, [1, 0, 0, 0, 0, 0]));
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        g.state.priority = p;
        let wolf = fixture(&mut g, p, "@token/hob-wolf", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        let shock = fixture(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Card(wolf)));
        finish(&mut g);
        assert_eq!(g.object_view(wolf).zone, Zone::Outside);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert_eq!(g.object_view(source).zone, Zone::Battlefield);
    }
}
