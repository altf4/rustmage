//! New virtual graveyard donors must be visible before actual mana payment.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};
#[test]
fn thranduil_funding_virtual_elf_sacrifice_unlocks_mana_without_live_game_fallback() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let t = named(&mut g, p, "Thranduil, the Elvenking", Zone::Battlefield);
        let elf = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        let ring = named(&mut g, p, "Lotus Ring", Zone::Battlefield);
        let grantor = g.object_ref(ring);
        assert!(g.attach_equipment(grantor, g.object_ref(elf)));
        g.state.players[p.index()].mana = [0; 6];
        let goal = FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, 4, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        assert!(g.can_fund(goal.clone()));
        let saved = g.snapshot();
        let world = FundingState::from_game(&g, &goal);
        let step = world
            .moves(&goal)
            .into_iter()
            .find(|s| {
                world.resources[s.resource].id == elf
                    && s.grantor == Some(grantor)
                    && s.production == [0, 0, 0, 0, 3, 0]
            })
            .expect("sacrifice mana source");
        let mut observed = false;
        world.visit_applied(&step, &mut |next| {
            assert_eq!(next.object(elf).zone, Zone::Graveyard);
            assert_eq!(g.object_view(elf).zone, Zone::Battlefield);
            let ability = borrowed_activations::abilities(next, t)
                .into_iter()
                .find(|a| a.donor.card == elf && a.ability == u16::MAX)
                .expect("new virtual donor");
            assert_eq!(ability.zone, Zone::Graveyard);
            assert_eq!(ability.donor, next.reference(elf));
            assert!(
                next.moves(&goal)
                    .iter()
                    .any(|s| next.resources[s.resource].id == t
                        && s.borrowed.is_some_and(|b| b.donor == ability.donor)
                        && s.production == [0, 0, 0, 0, 1, 0])
            );
            observed = true;
            true
        });
        assert!(observed);
        assert_eq!(g.snapshot(), saved);
        drop(world);
        g.apply(
            p,
            Action::TapGrantedMana {
                card: elf,
                grantor,
                production: [0, 0, 0, 0, 3, 0],
            },
        )
        .unwrap();
        assert_eq!(g.object_view(elf).zone, Zone::Graveyard);
        let graveyard = g.object_ref(elf);
        g.apply(
            p,
            Action::AnnounceGraveyardBorrowedMana {
                card: t,
                graveyard,
                ability: u16::MAX,
                counter_payment: Counters::default(),
                production: [0, 0, 0, 0, 1, 0],
            },
        )
        .unwrap();
        while g.state.announcing.is_some() {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .expect("zero payment");
            g.apply(g.actor().unwrap(), a).unwrap();
        }
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 4, 0]);
        assert!(g.object_view(t).tapped);
    }
}
