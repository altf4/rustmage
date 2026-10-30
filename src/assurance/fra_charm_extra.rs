//! Additional modal-charm interactions and legality checks.
use super::big_helpers::{cast, checked, settle, setup};
use super::*;

fn candidate(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = put(g, p, "Forest", Zone::Hand);
    g.state.objects[id as usize].card = Card::from_name(name).unwrap();
    id
}

#[test]
fn vigorbloom_protection_requires_control_at_targeting_and_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = candidate(&mut g, p, "Vigorbloom Charm");
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseMode { mode: 0 });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, payment);
        let target = g.object_ref(own);
        g.change_controller(target, p.other());
        let checkpoint = g.snapshot();
        settle(&mut g);
        assert!(
            !g.keywords(own)
                .intersects(Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE))
        );
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        g.restore(&checkpoint).unwrap();
        settle(&mut g);
        assert!(
            !g.keywords(own)
                .intersects(Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE))
        );
    }
}

#[test]
fn konstrari_mana_pays_a_subsequent_three_generic_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let charm = candidate(&mut g, p, "Konstrari Charm");
        let artifact = put(&mut g, p, "Lotus Ring", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 1, 0];
        cast(&mut g, charm, Some(2), &[]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 3]);
        cast(&mut g, artifact, None, &[]);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert!(g.on_battlefield(artifact));
    }
}

#[test]
fn stingerquill_cadet_has_printed_name_types_colors_and_haste() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let charm = candidate(&mut g, p, "Stingerquill Charm");
        cast(&mut g, charm, Some(2), &[]);
        settle(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(i, o)| {
                (o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.card == Card::test_index(5298))
                .then_some(i as CardId)
            })
            .unwrap();
        let view = g.object_view(token);
        assert!(g.state.objects[token as usize].is_token());
        assert_eq!(view.card.name(), "Cadet");
        assert_eq!(view.kinds, 1);
        assert_eq!(view.colors, 0);
        assert!(view.has_subtype("Wizard"));
        assert!(view.has_subtype("Soldier"));
        assert_eq!(stats(&g, token), (2, 2));
        assert!(g.keywords(token).contains(Keywords::HASTE));
        g.finish_cleanup();
        assert!(!g.keywords(token).contains(Keywords::HASTE));
    }
}

#[test]
fn fatehold_bonus_keeps_resolved_recipients_across_control_and_new_entries() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let old = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let foreign = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let charm = candidate(&mut g, p, "Fatehold Charm");
        cast(&mut g, charm, Some(2), &[]);
        settle(&mut g);
        assert_eq!(stats(&g, old), (3, 4));
        let old_ref = g.object_ref(old);
        let foreign_ref = g.object_ref(foreign);
        g.change_controller(old_ref, p.other());
        g.change_controller(foreign_ref, p);
        let new = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert_eq!(stats(&g, old), (3, 4));
        assert_eq!(stats(&g, foreign), (2, 2));
        assert_eq!(stats(&g, new), (2, 2));
        g.finish_cleanup();
        assert_eq!(stats(&g, old), (2, 2));
    }
}
