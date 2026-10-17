//! Dynamic production must use overlay presence, including nonmana Elf witnesses.
use super::*;
use crate::engine::ability_timeline_tests::{named, setup};
#[test]
fn fdn_six_scaled_mana_overlay_counts_noncreature_elves_and_removes_witnesses() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        let arch = named(&mut g, p, "Elvish Archdruid", Zone::Battlefield);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let kindred = named(&mut g, p, "Morcant's Eyes", Zone::Battlefield);
        named(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 3,
            reserve: None,
            source: None,
        };
        let mut world = FundingState::from_game(&g, &goal);
        let production = g.state.objects[arch as usize].card.activations()[0]
            .mana_production()
            .unwrap();
        assert_eq!(
            production.in_world(&world, arch),
            ManaProduction::Fixed([0, 0, 0, 0, 3, 0])
        );
        assert!(
            world
                .moves(&goal)
                .iter()
                .any(|m| world.resources[m.resource].id == arch && m.production[4] == 3)
        );
        let witness = world
            .resources
            .iter_mut()
            .find(|r| r.id == kindred)
            .expect("nonmana Elf is an owned overlay resource");
        witness.object.zone = Zone::Graveyard;
        assert_eq!(
            production.in_world(&world, arch),
            ManaProduction::Fixed([0, 0, 0, 0, 2, 0])
        );
        let witness = world.resources.iter_mut().find(|r| r.id == elf).unwrap();
        witness.object.controller = p.other();
        assert_eq!(
            production.in_world(&world, arch),
            ManaProduction::Fixed([0, 0, 0, 0, 1, 0])
        );
        assert_eq!(
            production.in_world(&g, arch),
            ManaProduction::Fixed([0, 0, 0, 0, 3, 0])
        );
        assert!(
            world.total_mana_upper() < 100,
            "finite current-world bound must not inflate X domains"
        );
    }
}
