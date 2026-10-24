//! Private funding overlay checks: no speculative basic-land witness may leak.
use super::*;
use crate::engine::assurance::big_helpers::{checked, named, setup};

fn goal(p: Player, color: usize, amount: u32) -> FundingGoal {
    let mut required = [0; 6];
    required[color] = amount;
    FundingGoal {
        actor: p,
        required,
        generic: 0,
        reserve: None,
        source: None,
    }
}
#[test]
fn msh_conditional_land_funding_tracks_departure_control_and_entry_keys() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [0; 6];
        let old = named(&mut g, p, "Dark Fortress", Zone::Battlefield);
        g.state.entered_this_turn = None; // Preserve a genuinely older source.
        let fresh = named(&mut g, p, "Dark Fortress", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: fresh });
        let basic = named(&mut g, p, "Forest", Zone::Battlefield);
        g.state.objects[basic as usize].tapped = true;
        let funding_goal = goal(p, 2, 2);
        let before = g.snapshot();
        let world = FundingState::from_game(&g, &funding_goal);
        assert!(world.clone().reachable(&funding_goal, &mut HashSet::new()));
        let mut departed = world.clone();
        departed
            .resources
            .iter_mut()
            .find(|r| r.id == basic)
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        assert!(
            !departed
                .clone()
                .reachable(&funding_goal, &mut HashSet::new())
        );
        let mut stolen = world.clone();
        stolen
            .resources
            .iter_mut()
            .find(|r| r.id == basic)
            .unwrap()
            .object
            .controller = p.other();
        assert!(!stolen.clone().reachable(&funding_goal, &mut HashSet::new()));
        assert!(departed.guard_available(
            ActivationGuard::SourceEnteredOrControlsBasicLand,
            fresh,
            p
        ));
        assert!(!departed.guard_available(
            ActivationGuard::SourceEnteredOrControlsBasicLand,
            old,
            p
        ));
        let mut a = departed.clone();
        a.resources
            .iter_mut()
            .find(|r| r.id == old)
            .unwrap()
            .object
            .tapped = true;
        let mut b = departed.clone();
        b.resources
            .iter_mut()
            .find(|r| r.id == fresh)
            .unwrap()
            .object
            .tapped = true;
        assert!(a.clone().reachable(&goal(p, 2, 1), &mut HashSet::new()));
        assert!(!b.clone().reachable(&goal(p, 2, 1), &mut HashSet::new()));
        assert!(
            a.key(&funding_goal) != b.key(&funding_goal),
            "same definition is not interchangeable across entry histories"
        );
        assert_eq!(g.snapshot(), before);
    }
}
#[test]
fn msh_conditional_land_funding_observes_immediate_virtual_entry_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Hidden Lair", Zone::Graveyard);
        let funding_goal = goal(p, 1, 1);
        let mut world = FundingState::from_game(&g, &funding_goal);
        world.prepare_entry_projection();
        let resource = world.resources.iter_mut().find(|r| r.id == source).unwrap();
        resource.object.zone = Zone::Battlefield;
        resource.object.generation += 1;
        let current = world.reference(source);
        world.entry_projection.as_mut().unwrap().order.push(current);
        assert!(world.guard_available(
            ActivationGuard::SourceEnteredOrControlsBasicLand,
            source,
            p
        ));
        world
            .resources
            .iter_mut()
            .find(|r| r.id == source)
            .unwrap()
            .object
            .generation += 1;
        assert!(!world.guard_available(
            ActivationGuard::SourceEnteredOrControlsBasicLand,
            source,
            p
        ));
    }
}

#[test]
fn msh_conditional_land_funding_retains_ordinary_basic_land_equivalence() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let old = named(&mut g, p, "Forest", Zone::Battlefield);
        g.state.entered_this_turn = None;
        let fresh = named(&mut g, p, "Forest", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: fresh });
        // Isolate entry history from the preexisting summoning-sickness key.
        g.state.objects[fresh as usize].summoning_sick = false;
        let goal = goal(p, 4, 1);
        let mut a = FundingState::from_game(&g, &goal);
        a.resources
            .iter_mut()
            .find(|r| r.id == old)
            .unwrap()
            .object
            .tapped = true;
        let mut b = FundingState::from_game(&g, &goal);
        b.resources
            .iter_mut()
            .find(|r| r.id == fresh)
            .unwrap()
            .object
            .tapped = true;
        assert!(a.key(&goal) == b.key(&goal));
    }
}
