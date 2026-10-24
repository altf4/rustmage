//! History-qualified group keywords in independent funding overlays.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};
fn goal(p: Player) -> FundingGoal {
    FundingGoal {
        actor: p,
        required: [0; 6],
        generic: 0,
        reserve: None,
        source: None,
    }
}
fn hex(world: &FundingState<'_>, id: CardId) -> bool {
    characteristic_query::CharacteristicQuery::in_world(world)
        .keywords(id)
        .contains(Keywords::HEXPROOF)
}
#[test]
fn placement_group_speculative_history_controller_and_source_departure_are_owned() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let kid = named(&mut g, p, "Kid Loki", Zone::Battlefield);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let reference = g.object_ref(elf);
        let before = g.snapshot();
        let goal = goal(p);
        let mut world = FundingState::from_game(&g, &goal);
        let source = world
            .resources
            .iter()
            .position(|r| r.id == kid)
            .expect("nonmana grant source retained");
        let target = world.resources.iter().position(|r| r.id == elf).unwrap();
        assert!(!hex(&world, elf));
        let key = world.key(&goal);
        counter_placement_history::record(&mut world.counter_placement_history, reference, p, true);
        assert!(hex(&world, elf));
        assert!(world.key(&goal) != key);
        assert!(
            world
                .keywords(&world.resources[target])
                .contains(Keywords::HEXPROOF)
        );
        let mut branch = world.clone();
        branch.resources[source].object.zone = Zone::Graveyard;
        assert!(!hex(&branch, elf));
        assert!(hex(&world, elf));
        branch = world.clone();
        branch.resources[target].object.controller = p.other();
        assert!(!hex(&branch, elf));
        branch = world.clone();
        branch.resources[target].object.generation += 1;
        assert!(!hex(&branch, elf));
        assert_eq!(g.snapshot(), before);
        assert!(!g.keywords(elf).contains(Keywords::HEXPROOF));
    }
}
#[test]
fn placement_group_speculative_source_blanking_and_layer_evaluator_use_overlay() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let kid = named(&mut g, p, "Kid Loki", Zone::Battlefield);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        g.effect_counter(
            elf,
            CounterKind::PlusOne,
            1,
            replacements::ResumeMode::Pending,
        );
        let moon = named(&mut g, p, "Imprisoned in the Moon", Zone::Battlefield);
        g.state.objects[moon as usize].attached_to = Some(g.object_ref(kid));
        let before = g.snapshot();
        let goal = goal(p);
        let mut world = FundingState::from_game(&g, &goal);
        assert!(!hex(&world, elf));
        world.prepare_entry_projection();
        let index = world.resources.iter().position(|r| r.id == moon).unwrap();
        world.resources[index].object.zone = Zone::Graveyard;
        assert!(hex(&world, elf));
        assert_eq!(g.snapshot(), before);
        assert!(!g.keywords(elf).contains(Keywords::HEXPROOF));
    }
}
