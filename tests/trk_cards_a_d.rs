//! Star Trek A–D card behavior through paid legal actions.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn owned(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}

fn drain(game: &mut Game) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("resolution budget");
}

fn mana(color: usize) -> Mana {
    let mut output = [0; 6];
    output[color] = 1;
    output
}

#[test]
fn three_planets_enter_tapped_gain_life_and_make_only_printed_colors() {
    for (name, colors) in [
        ("Andoria", [1, 3]),
        ("Bajor", [3, 0]),
        ("Cardassia Prime", [0, 2]),
    ] {
        for player in [Player::P0, Player::P1] {
            for color in colors {
                let mut game = scenario(player);
                game.state.players[player.index()].mana = [0; 6];
                let land = owned(&mut game, player, name, Zone::Hand);
                act(&mut game, Action::PlayLand { card: land });
                assert!(game.object_view(land).tapped, "{name}");
                assert_eq!(game.state.players[player.index()].life, 20);
                drain(&mut game);
                assert_eq!(game.state.players[player.index()].life, 21, "{name}");
                reject(
                    &mut game,
                    Action::ActivateMana {
                        card: land,
                        ability: 0,
                        payment: [0; 6],
                        counter_payment: Counters::default(),
                        production: mana(color),
                    },
                );
                game.untap_objects(&[land], untap_rules::UntapCause::Effect);
                act(
                    &mut game,
                    Action::ActivateMana {
                        card: land,
                        ability: 0,
                        payment: [0; 6],
                        counter_payment: Counters::default(),
                        production: mana(color),
                    },
                );
                assert_eq!(
                    game.state.players[player.index()].mana,
                    mana(color),
                    "{name}"
                );
            }
        }
    }
}

#[test]
fn assembly_specialist_creates_exactly_one_untapped_borg_artifact_creature() {
    let mut game = scenario(Player::P0);
    let specialist = owned(&mut game, Player::P0, "Assembly Specialist", Zone::Hand);
    cast(&mut game, specialist, None);
    drain(&mut game);
    let tokens: Vec<_> = game
        .state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.controller == Player::P0
                && object.is_token())
            .then_some(id as CardId)
        })
        .collect();
    assert_eq!(tokens.len(), 1);
    assert_eq!(game.object_view(tokens[0]).card, Card(6007));
    assert_eq!(stats(&game, tokens[0]), (1, 1));
    assert!(!game.object_view(tokens[0]).tapped);
}

#[test]
fn assault_drone_and_subadjunct_grant_only_controlled_artifact_creatures() {
    let mut game = scenario(Player::P0);
    let drone = owned(&mut game, Player::P0, "Assault Drone", Zone::Battlefield);
    let adjunct = owned(
        &mut game,
        Player::P0,
        "Command Subadjunct",
        Zone::Battlefield,
    );
    let specialist = owned(
        &mut game,
        Player::P0,
        "Assembly Specialist",
        Zone::Battlefield,
    );
    let ordinary = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let enemy = owned(
        &mut game,
        Player::P1,
        "Assembly Specialist",
        Zone::Battlefield,
    );
    assert_eq!(stats(&game, drone), (3, 4));
    assert_eq!(stats(&game, adjunct), (3, 1));
    assert_eq!(stats(&game, specialist), (2, 1));
    assert_eq!(stats(&game, ordinary), (2, 2));
    assert_eq!(stats(&game, enemy), (1, 1));
    assert!(game.keywords(drone).contains(Keywords::VIGILANCE));
    assert!(game.keywords(specialist).contains(Keywords::VIGILANCE));
    assert!(!game.keywords(ordinary).contains(Keywords::VIGILANCE));
    assert!(!game.keywords(enemy).contains(Keywords::VIGILANCE));
    game.move_object(adjunct, Zone::Graveyard);
    assert!(!game.keywords(specialist).contains(Keywords::VIGILANCE));
    game.move_object(drone, Zone::Graveyard);
    assert_eq!(stats(&game, specialist), (1, 1));
}

#[test]
fn data_new_life_grows_on_each_own_draw_but_not_opponents_draw() {
    let mut game = scenario(Player::P0);
    let data = owned(&mut game, Player::P0, "Data, New Life", Zone::Battlefield);
    let draw_spell = put(&mut game, Player::P0, "Think Twice", Zone::Hand);
    cast(&mut game, draw_spell, None);
    drain(&mut game);
    assert_eq!(game.object_view(data).counters.plus_one, 1);
    game.draw_many(Player::P1, 1);
    drain(&mut game);
    assert_eq!(game.object_view(data).counters.plus_one, 1);
}
