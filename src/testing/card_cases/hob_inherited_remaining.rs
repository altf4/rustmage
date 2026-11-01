//! Focused printed outcomes; shared engine invariants live in core tests.
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
    {
        let p = Player::P0;
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
fn hob_inherited_remaining_hole_four_hand_discard_halflingcycling_reveal_and_fail_to_find() {
    {
        let p = Player::P0;
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
fn hob_inherited_remaining_opportunist_paid_exile_public_land_and_normal_paid_spell_timing() {
    {
        let p = Player::P0;
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
fn hob_inherited_remaining_sackville_optional_other_creature_artifact_token_owner_and_draw_treasure()
 {
    {
        let p = Player::P0;
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
            let token_sac = matches!(mode, 3..=5);
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
