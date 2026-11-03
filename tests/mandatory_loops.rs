use rustmage::CATALOGUE_VERSION;
use rustmage_core::engine::*;
mod support;
use support::*;

fn cycle_for(player: Player) -> Game {
    assert!(!CATALOGUE_VERSION.is_empty());
    let mut g = scenario(player);
    let snare = put(&mut g, player.other(), "Perilous Snare", Zone::Battlefield);
    let cage = put(&mut g, player, "Pinnacle Starcage", Zone::Exile);
    g.state.objects[snare as usize].generation = 7;
    g.state.objects[cage as usize].generation = 11;
    // put() also records this exact incarnation in entry history.
    for object in g.state.entered_this_turn.as_mut().unwrap().iter_mut() {
        if object.card == snare {
            object.generation = 7;
        }
    }
    for stamp in &mut g.state.roles.as_mut().unwrap().ability_timestamps {
        if stamp.source.card == snare {
            stamp.source.generation = 7;
        }
    }

    g.state.exile_links.push(ExileLink {
        source: g.object_ref(snare),
        exiled: g.object_ref(cage),
        return_zone: Zone::Battlefield,
    });
    let cell = put(&mut g, player, "Solitary Cell", Zone::Hand);
    cast(&mut g, cell, None);
    for p in &mut g.state.players {
        p.mana = [0; 6];
    }
    g
}

fn cycle() -> Game {
    cycle_for(Player::P1)
}

fn advance(g: &mut Game, indexed: bool) {
    let actions = g.legal_actions();
    let index = actions
        .iter()
        .position(|a| matches!(a, Action::Pass | Action::ChooseTarget { .. }))
        .unwrap();
    if indexed {
        g.apply_index(g.actor().unwrap(), g.revision(), index)
            .unwrap();
    } else {
        act(g, actions[index].clone());
    }
}

#[test]
fn mandatory_exile_loop_draws_raw_indexed_and_restored_branches() {
    for player in [Player::P0, Player::P1] {
        let mut g = cycle_for(player);
        let saved = g.snapshot();
        for indexed in [false, true] {
            let mut branch = g.clone();
            for _ in 0..48 {
                if branch.outcome().is_some() {
                    break;
                }
                advance(&mut branch, indexed);
            }
            assert_eq!(branch.outcome(), Some(Outcome::Draw));
            assert_eq!(g.snapshot(), saved);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn optional_target_and_observer_never_force_a_draw() {
    for other in ["Ajani's Pridemate", "Pests of Honor"] {
        let mut g = cycle();
        put(&mut g, Player::P0, other, Zone::Battlefield);
        for _ in 0..80 {
            advance(&mut g, false);
            assert_eq!(g.outcome(), None);
        }
    }
}

#[test]
fn optional_instant_and_rejected_indices_preserve_the_game() {
    let mut g = cycle();
    put(&mut g, Player::P0, "Disenchant", Zone::Hand);
    g.state.players[0].mana = [1, 0, 0, 0, 0, 1];
    for _ in 0..64 {
        let saved = g.snapshot();
        let revision = g.revision();
        assert!(
            g.apply_index(g.actor().unwrap(), revision.wrapping_sub(1), 0)
                .is_err()
        );
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), revision);
        advance(&mut g, true);
        assert_eq!(g.outcome(), None);
    }
}

#[test]
fn stale_unmapped_entry_history_declines_detection() {
    let mut g = cycle();
    let source = g.state.exile_links[0].source.card;
    g.state.entered_this_turn.as_mut().unwrap().push(ObjectRef {
        card: source,
        generation: 0,
    });
    for _ in 0..64 {
        advance(&mut g, false);
        assert_eq!(g.outcome(), None);
    }
}

#[test]
fn unplaceable_entry_damage_order_does_not_hide_mandatory_cycle() {
    for player in [Player::P0, Player::P1] {
        let mut g = cycle_for(player);
        put(
            &mut g,
            player,
            "Spring-Loaded Sawblades // Bladewheel Chariot",
            Zone::Battlefield,
        );
        let saved = g.snapshot();
        for indexed in [false, true] {
            let mut branch = g.clone();
            for _ in 0..64 {
                if branch.outcome().is_some() {
                    break;
                }
                let actions = branch.legal_actions();
                let index = actions
                    .iter()
                    .position(|a| {
                        matches!(
                            a,
                            Action::Pass | Action::ChooseTarget { .. } | Action::PutTrigger { .. }
                        )
                    })
                    .unwrap();
                if indexed {
                    branch
                        .apply_index(branch.actor().unwrap(), branch.revision(), index)
                        .unwrap();
                } else {
                    act(&mut branch, actions[index].clone());
                }
            }
            assert_eq!(branch.outcome(), Some(Outcome::Draw));
            assert_eq!(g.snapshot(), saved);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn both_explicit_trigger_orders_draw_without_mutating_saved_branches() {
    for player in [Player::P0, Player::P1] {
        let mut g = cycle_for(player);
        put(
            &mut g,
            player,
            "Spring-Loaded Sawblades // Bladewheel Chariot",
            Zone::Battlefield,
        );
        put(&mut g, player, "Disenchant", Zone::Hand);
        g.state.players[player.index()].mana = [1, 0, 0, 0, 0, 1];
        for _ in 0..32 {
            if matches!(g.state.pending, Pending::OrderTriggers { .. }) {
                break;
            }
            advance(&mut g, false);
            assert_eq!(g.outcome(), None);
        }
        assert!(matches!(g.state.pending, Pending::OrderTriggers { .. }));
        for p in &mut g.state.players {
            p.mana = [0; 6];
        }
        let saved = g.snapshot();
        let orders: Vec<_> = g
            .legal_actions()
            .into_iter()
            .filter(|a| matches!(a, Action::PutTrigger { .. }))
            .collect();
        assert_eq!(orders.len(), 2);
        for first in orders {
            for indexed in [false, true] {
                let mut branch = g.clone();
                let index = branch
                    .legal_actions()
                    .iter()
                    .position(|a| *a == first)
                    .unwrap();
                if indexed {
                    branch
                        .apply_index(branch.actor().unwrap(), branch.revision(), index)
                        .unwrap();
                } else {
                    act(&mut branch, first.clone());
                }
                assert_eq!(branch.outcome(), Some(Outcome::Draw));
                assert_eq!(g.snapshot(), saved);
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn available_damage_target_remains_a_real_choice() {
    let mut g = cycle();
    let blade = put(
        &mut g,
        Player::P1,
        "Spring-Loaded Sawblades // Bladewheel Chariot",
        Zone::Battlefield,
    );
    let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.state.objects[bear as usize].tapped = true;
    for _ in 0..32 {
        if matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            break;
        }
        advance(&mut g, false);
        assert_eq!(g.outcome(), None);
    }
    let trigger = g
        .state
        .ordering_triggers
        .iter()
        .find(|i| i.source.unwrap().object.card == blade)
        .unwrap()
        .id;
    act(&mut g, Action::PutTrigger { id: trigger });
    assert_eq!(g.outcome(), None);
    assert!(g.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Card(bear)
    }));
}

#[test]
fn funded_instant_escape_stays_playable_with_observer() {
    let mut g = cycle();
    put(
        &mut g,
        Player::P1,
        "Spring-Loaded Sawblades // Bladewheel Chariot",
        Zone::Battlefield,
    );
    let instant = put(&mut g, Player::P0, "Disenchant", Zone::Hand);
    g.state.players[0].mana = [1, 0, 0, 0, 0, 1];
    let mut seen_escape = false;
    for _ in 0..64 {
        if g.legal_actions()
            .iter()
            .any(|a| matches!(a,Action::AnnounceCast{card} if *card==instant))
        {
            seen_escape = true;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass | Action::ChooseTarget { .. } | Action::PutTrigger { .. }
                )
            })
            .unwrap();
        act(&mut g, action);
        assert_eq!(g.outcome(), None);
    }
    assert!(seen_escape);
}

#[test]
fn funded_sorcery_speed_craft_is_not_an_escape_during_the_cycle() {
    let mut g = cycle();
    put(
        &mut g,
        Player::P1,
        "Spring-Loaded Sawblades // Bladewheel Chariot",
        Zone::Battlefield,
    );
    g.state.players[1].mana = [1, 0, 0, 0, 0, 3];
    for _ in 0..64 {
        if g.outcome().is_some() {
            break;
        }
        let actions = g.legal_actions();
        assert!(
            !actions
                .iter()
                .any(|a| matches!(a, Action::AnnounceActivate { .. } | Action::Activate { .. }))
        );
        let action = actions
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass | Action::ChooseTarget { .. } | Action::PutTrigger { .. }
                )
            })
            .unwrap();
        act(&mut g, action);
    }
    assert_eq!(g.outcome(), Some(Outcome::Draw));
}

#[test]
fn funded_starcage_activation_escape_is_not_overridden_by_the_proof() {
    let mut g = cycle();
    put(
        &mut g,
        Player::P1,
        "Spring-Loaded Sawblades // Bladewheel Chariot",
        Zone::Battlefield,
    );
    // Starcage's instant-speed escape costs 6WW. Sawblades craft alone costs 3W.
    g.state.players[1].mana = [2, 0, 0, 0, 0, 6];
    let mut seen_escape = false;
    for _ in 0..64 {
        let actions = g.legal_actions();
        seen_escape |= actions
            .iter()
            .any(|a| matches!(a, Action::AnnounceActivate { .. } | Action::Activate { .. }));
        let action = actions
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass | Action::ChooseTarget { .. } | Action::PutTrigger { .. }
                )
            })
            .unwrap();
        act(&mut g, action);
        assert_eq!(g.outcome(), None);
    }
    assert!(seen_escape);
}
