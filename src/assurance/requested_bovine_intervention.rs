//! Bovine Intervention uses the target's controller even if destruction fails.
use super::big_helpers::{cast, checked, settle};
use super::*;

fn oxen(g: &Game, player: Player) -> usize {
    let ox = Card::from_name("@token/msc-bovine-ox").unwrap();
    g.state
        .objects
        .iter()
        .filter(|o| o.card == ox && o.zone == Zone::Battlefield && o.controller == player)
        .count()
}

#[test]
fn bovine_intervention_controls_token_and_handles_indestructible() {
    for player in [Player::P0, Player::P1] {
        for indestructible in [false, true] {
            let mut g = scenario(player);
            let spell = put(&mut g, player, "Bovine Intervention", Zone::Hand);
            let target = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
            if indestructible {
                g.state.objects[target as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
            }
            cast(&mut g, spell, None, &[Target::Card(target)]);
            settle(&mut g);
            assert_eq!(
                g.object_view(target).zone,
                if indestructible {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(oxen(&g, player.other()), 1);
            assert_eq!(oxen(&g, player), 0);
            let ox = g
                .state
                .objects
                .iter()
                .position(|o| {
                    o.card == Card::from_name("@token/msc-bovine-ox").unwrap()
                        && o.zone == Zone::Battlefield
                })
                .unwrap() as CardId;
            assert_eq!((g.stats(ox).power, g.stats(ox).toughness), (2, 2));
            assert!(g.has_subtype(ox, "Ox"));
            assert_eq!(g.object_view(ox).card.colors(), 1);
        }
    }
}

#[test]
fn bovine_intervention_validates_target_and_lost_target_produces_no_token() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let spell = put(&mut g, player, "Bovine Intervention", Zone::Hand);
        let land = put(&mut g, player.other(), "Forest", Zone::Battlefield);
        let target = put(&mut g, player.other(), "Short Bow", Zone::Battlefield);
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        g.move_object(target, Zone::Exile);
        g.move_object(target, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert_eq!(oxen(&g, player.other()), 0);
    }
}

#[test]
fn bovine_intervention_uses_controller_not_owner() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let spell = put(&mut g, player, "Bovine Intervention", Zone::Hand);
        let target = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[target as usize].controller = player.other();
        cast(&mut g, spell, None, &[Target::Card(target)]);
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Graveyard);
        assert_eq!(oxen(&g, player.other()), 1);
        assert_eq!(oxen(&g, player), 0);
    }
}
