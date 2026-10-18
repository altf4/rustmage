//! Whole-body casting-origin rules, with paid actions and owned replay.
use super::ability_timeline_tests::{checked, settle, setup};
use super::tests::spawn;
use super::*;
fn put(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn finish(g: &mut Game) {
    for _ in 0..40 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap_or_else(|| panic!("unexpected {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("unfinished announcement");
}
#[test]
fn cast_origins_vizier_private_live_top_paid_cast_and_hidden_next_during_announcement() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let vizier = put(&mut g, p, "Vizier of the Menagerie", Zone::Battlefield);
        let next = put(&mut g, p, "Shock", Zone::Library);
        let lion = put(&mut g, p, "Savannah Lions", Zone::Library);
        assert_eq!(
            g.observe(p).known_library_top[p.index()].unwrap().object,
            g.object_ref(lion)
        );
        assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        checked(&mut g, Action::AnnounceCast { card: lion });
        assert!(g.observe(p).known_library_top[p.index()].is_none());
        finish(&mut g);
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().payment,
            [0, 0, 0, 0, 0, 1]
        );
        assert_eq!(
            g.observe(p).known_library_top[p.index()].unwrap().object,
            g.object_ref(next)
        );
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: next })
        );
        settle(&mut g);
        assert_eq!(g.state.objects[lion as usize].zone, Zone::Battlefield);
        g.move_object(vizier, Zone::Graveyard);
        assert!(g.observe(p).known_library_top[p.index()].is_none());
    }
}
#[test]
fn cast_origins_vizier_all_origins_creature_only_and_timing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        put(&mut g, p, "Vizier of the Menagerie", Zone::Battlefield);
        let lion = put(&mut g, p, "Savannah Lions", Zone::Hand);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: lion })
        );
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: shock })
        );
        g.state.active = p.other();
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: lion })
        );
        let flash = put(&mut g, p, "Brineborn Cutthroat", Zone::Library);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: flash })
        );
        checked(&mut g, Action::AnnounceCast { card: flash });
        finish(&mut g);
        settle(&mut g);
        assert_eq!(g.state.objects[flash as usize].zone, Zone::Battlefield);
    }
}
#[test]
fn cast_origins_vizier_lost_ability_and_required_provider_reservation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let vizier = put(&mut g, p, "Vizier of the Menagerie", Zone::Battlefield);
        let lion = put(&mut g, p, "Savannah Lions", Zone::Hand);
        let land = put(&mut g, p, "Wastes", Zone::Battlefield);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: lion })
        );
        let turn = put(&mut g, p.other(), "Witness Protection", Zone::Hand);
        // Normal public entry attaches the ability-removing Aura.
        g.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 0];
        g.state.active = p.other();
        g.state.priority = p.other();
        super::ability_timeline_tests::cast(&mut g, turn, None, &[Target::Card(vizier)]);
        settle(&mut g);
        g.state.active = p;
        g.state.priority = p;
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceCast { card: lion })
        );
        assert_eq!(g.state.objects[land as usize].zone, Zone::Battlefield);
    }
}
