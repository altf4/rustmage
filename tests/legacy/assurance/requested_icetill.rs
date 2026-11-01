//! Paid Icetill land plays, landfall and source-departure behavior.
use super::*;

#[test]
fn icetill_grants_extra_land_play_and_graveyard_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let explorer = put(&mut g, p, "Icetill Explorer", Zone::Hand);
        big_helpers::cast(&mut g, explorer, None, &[]);
        big_helpers::settle(&mut g);
        let first = put(&mut g, p, "Forest", Zone::Hand);
        let second = put(&mut g, p, "Island", Zone::Graveyard);
        let third = put(&mut g, p, "Mountain", Zone::Hand);
        let foreign = put(&mut g, p.other(), "Plains", Zone::Graveyard);
        reject(&mut g, Action::PlayLand { card: foreign });
        let library_before = g.state.players[p.index()].library.len();
        big_helpers::checked(&mut g, Action::PlayLand { card: first });
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), library_before - 1);
        let before = g.snapshot();
        let mut restored = g.clone();
        big_helpers::checked(&mut g, Action::PlayLand { card: second });
        big_helpers::settle(&mut g);
        assert_eq!(g.object_view(second).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.index()].library.len(), library_before - 2);
        reject(&mut g, Action::PlayLand { card: third });
        restored.restore(&before).unwrap();
        big_helpers::checked(&mut restored, Action::PlayLand { card: second });
        big_helpers::settle(&mut restored);
        assert_eq!(restored.snapshot(), g.snapshot());
    }
}

#[test]
fn icetill_landfall_survives_departure_but_graveyard_permission_does_not() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let explorer = put(&mut g, p, "Icetill Explorer", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Forest", Zone::Hand);
        let land = put(&mut g, p, "Forest", Zone::Hand);
        let grave = put(&mut g, p, "Island", Zone::Graveyard);
        let before = g.state.players[p.index()].library.len();
        g.move_object(enemy, Zone::Battlefield);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before);
        big_helpers::checked(&mut g, Action::PlayLand { card: land });
        g.move_object(explorer, Zone::Graveyard);
        big_helpers::settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before - 1);
        reject(&mut g, Action::PlayLand { card: grave });
    }
}
