//! The activation-only creature-card threshold reads owned funding overlays.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};
#[test]
fn msh_punch_zola_funding_guard_uses_actual_owner_non_tokens_and_virtual_departures() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Arnim Zola, Bio-Fanatic", Zone::Battlefield);
        let a = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        let b = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        named(&mut g, p.other(), "Hill Giant", Zone::Graveyard);
        named(&mut g, p, "Shock", Zone::Graveyard);
        named(
            &mut g,
            p,
            "@token/cf0227c0984ee9114e6c311f",
            Zone::Graveyard,
        );
        let secret = named(&mut g, p, "Shock", Zone::Hand);
        let guard = ActivationGuard::GraveyardCardsAtLeast {
            kind: 1,
            minimum: 2,
        };
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 3,
            reserve: None,
            source: None,
        };
        let before = g.snapshot();
        let world = FundingState::from_game(&g, &goal);
        assert!(!world.guard_available(guard, source, p));
        assert!(!g.activation_guard_available(guard, source, p));
        let mut visited = 0;
        world.cost_departures(&[g.object_ref(b)], Zone::Graveyard, &mut |departed| {
            visited += 1;
            assert!(departed.guard_available(guard, source, p));
            assert!(!departed.guard_available(guard, source, p.other()));
            // A card's stale battlefield controller does not define its graveyard.
            let mut transferred = departed.clone();
            transferred
                .resources
                .iter_mut()
                .find(|r| r.id == b)
                .unwrap()
                .object
                .controller = p.other();
            assert!(transferred.guard_available(guard, source, p));
            let mut exiled = departed.clone();
            let old = exiled
                .resources
                .iter()
                .find(|r| r.id == a)
                .map(|r| r.object.clone())
                .unwrap_or_else(|| g.state.objects[a as usize].clone());
            exiled.resources.push(Resource {
                id: a,
                attacking: false,
                object: old,
            });
            exiled
                .resources
                .iter_mut()
                .find(|r| r.id == a)
                .unwrap()
                .object
                .zone = Zone::Exile;
            assert!(!exiled.guard_available(guard, source, p));
            false
        });
        assert_eq!(visited, 1);
        assert_eq!(g.snapshot(), before);
        assert!(g.observe(p.other()).objects.iter().all(|o| o.id != secret));
    }
}
