use super::*;
use crate::engine::assurance::big_helpers::{named, setup};

#[test]
fn fast_sacrificed_mana_source_resets_incarnation_history_like_owned_move() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let ring = named(&mut g, p, "Lotus Ring", Zone::Battlefield);
        let grantor = g.object_ref(ring);
        assert!(g.attach_equipment(grantor, g.object_ref(id)));
        g.state.objects[id as usize].summoning_sick = false;
        g.state.objects[id as usize].dealt_combat_damage_this_incarnation = true;
        g.state.players[p.index()].mana = [0; 6];
        let production = [0, 0, 0, 0, 3, 0];
        let goal = FundingGoal {
            actor: p,
            required: production,
            generic: 0,
            reserve: None,
            source: None,
        };
        let saved = g.snapshot();
        let before = g.object_ref(id);
        let mut projected = FundingState::from_game(&g, &goal);
        let step = projected
            .moves(&goal)
            .into_iter()
            .find(|m| {
                projected.resources[m.resource].id == id
                    && m.grantor == Some(grantor)
                    && m.production == production
            })
            .unwrap();
        projected.apply(&step, false);
        let virtual_object = projected.resources[step.resource].object.clone();
        assert_eq!(virtual_object.zone, Zone::Graveyard);
        assert_ne!(virtual_object.generation, before.generation);
        assert!(!virtual_object.dealt_combat_damage_this_incarnation);
        drop(projected);
        assert_eq!(g.snapshot(), saved);
        let action = Action::TapGrantedMana {
            card: id,
            grantor,
            production,
        };
        g.apply(p, action.clone()).unwrap();
        assert_eq!(g.state.objects[id as usize], virtual_object);
        let after = g.snapshot();
        g.restore(&saved).unwrap();
        assert!(g.state.objects[id as usize].dealt_combat_damage_this_incarnation);
        g.apply(p, action).unwrap();
        assert_eq!(g.snapshot(), after);
    }
}
