//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

#[test]
fn gallia_rejects_activation_without_other_creature_or_mana() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Gallia, Tragic Host", Zone::Graveyard);
        let activate = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, activate.clone());
        let own = put(&mut g, p, "Bear Cub", Zone::Graveyard);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
        reject(&mut g, activate);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.object_view(own).zone, Zone::Graveyard);
    }
}
