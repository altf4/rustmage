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
        ability_origin: None,
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
