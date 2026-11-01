//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

#[test]
fn loot_projected_mana_funds_a_spell_using_one_chosen_color() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let loot = put(&mut g, player, "Loot, the Nexus", Zone::Battlefield);
        put(&mut g, player, "Campus Crier", Zone::Battlefield);
        let spell = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[loot as usize].summoning_sick = false;
        g.state.players[player.index()].mana = [0; 6];
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
        act(&mut g, Action::AnnounceCast { card: spell });
        let green = g.legal_actions().into_iter().find(|action| matches!(action,
            Action::ActivateMana { card, ability: 0, production: [0, 0, 0, 0, 2, 0], .. } if *card == loot)).unwrap();
        act(&mut g, green);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
    }
}

#[test]
fn loot_zero_creature_production_has_one_zero_mana_choice() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        // Loot's ability can be borrowed by a noncreature source.
        let source = put(&mut g, player, "Forest", Zone::Battlefield);
        let production = ManaProduction::DistinctCreaturePowers.in_world(&g, source);
        assert_eq!(production, ManaProduction::Fixed([0; 6]));
        let mut rows = Vec::new();
        production.visit(|mana| {
            rows.push(mana);
            false
        });
        assert_eq!(rows, vec![[0; 6]]);
    }
}
