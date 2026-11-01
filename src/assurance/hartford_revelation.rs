//! Jeskai Revelation's ordered target groups and prowess token behavior.
use super::big_helpers::{cast, settle, setup};
use super::*;

#[test]
fn revelation_returns_spell_or_permanent_and_completes_all_five_effects() {
    for player in [Player::P0, Player::P1] {
        for spell_target in [false, true] {
            let mut g = setup(player);
            let returned = if spell_target {
                let id = put(&mut g, player, "Quick Study", Zone::Hand);
                cast(&mut g, id, None, &[]);
                id
            } else {
                put(&mut g, player.other(), "Bear Cub", Zone::Battlefield)
            };
            let revelation = put(&mut g, player, "Jeskai Revelation", Zone::Hand);
            let hand = g.hand_count(player);
            cast(
                &mut g,
                revelation,
                None,
                &[
                    if spell_target {
                        Target::Spell(returned)
                    } else {
                        Target::Card(returned)
                    },
                    Target::Player(player.other()),
                ],
            );
            settle(&mut g);
            assert_eq!(g.object_view(returned).zone, Zone::Hand);
            assert_eq!(g.state.players[player.index()].life, 24);
            assert_eq!(g.state.players[player.other().index()].life, 16);
            assert_eq!(g.hand_count(player), hand + 1 + usize::from(spell_target));
            let monks: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(id, o)| {
                    (o.zone == Zone::Battlefield && o.controller == player && o.card == Card(5486))
                        .then_some(id as CardId)
                })
                .collect();
            assert_eq!(monks.len(), 2);
            for &id in &monks {
                assert_eq!(
                    g.stats(id),
                    Stats {
                        power: 1,
                        toughness: 1
                    }
                );
            }
            let study = put(&mut g, player, "Quick Study", Zone::Hand);
            cast(&mut g, study, None, &[]);
            settle(&mut g);
            for id in monks {
                assert_eq!(
                    g.stats(id),
                    Stats {
                        power: 2,
                        toughness: 2
                    }
                );
            }
        }
    }
}
