//! Actual owned departures and exact virtual resource removals share all projections.
use super::super::ability_timeline_tests::{cast, checked, settle, setup};
use super::super::ability_world::CharacteristicWorld;
use super::super::tests::spawn;
use super::*;
use crate::cards::CreatureType;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
#[test]
fn tagged_mana_funding_virtual_aura_departure_restores_name_colors_types_and_paid_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let guide = named(&mut g, p, "Scarecrow Guide", Zone::Battlefield);
        let witness = named(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, witness, None, &[Target::Card(guide)]);
        settle(&mut g);
        let court = named(&mut g, p, "Secluded Courtyard", Zone::Battlefield);
        g.set_entry_value(
            g.object_ref(court),
            EntryChoiceValue::CreatureType(CreatureType::from_name("Scarecrow").unwrap()),
        );
        g.state.players[p.index()].mana = [0; 6];
        checked(
            &mut g,
            Action::ActivateMana {
                card: court,
                ability: 1,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 1, 0, 0],
            },
        );
        let goal = FundingGoal {
            actor: p,
            required: [1, 0, 0, 0, 0, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let before = g.snapshot();
        let mut virtual_world = FundingState::from_game(&g, &goal);
        assert_eq!(
            CharacteristicWorld::name(&virtual_world, guide),
            "Legitimate Businessperson"
        );
        assert!(!virtual_world.reachable(&goal, &mut HashSet::new()));
        virtual_world
            .resources
            .iter_mut()
            .find(|r| r.id == witness)
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        assert!(virtual_world.reachable(&goal, &mut HashSet::new()));
        let mut child = {
            let parent = g.clone();
            parent.clone()
        };
        child.sacrifice_batch(&[(p, witness)]);
        child.state_based_actions();
        assert_eq!(
            CharacteristicWorld::name(&virtual_world, guide),
            child.object_view(guide).name()
        );
        assert_eq!(
            CharacteristicWorld::colors(&virtual_world, guide),
            child.object_view(guide).colors
        );
        assert_eq!(
            CharacteristicWorld::kinds(&virtual_world, guide),
            child.current_kinds(guide)
        );
        assert!(CharacteristicWorld::subtypes(&virtual_world, guide).has(
            CharacteristicWorld::object(&virtual_world, guide).card,
            "Scarecrow"
        ));
        assert!(!CharacteristicWorld::subtypes(&virtual_world, guide).has(
            CharacteristicWorld::object(&virtual_world, guide).card,
            "Citizen"
        ));
        checked(
            &mut child,
            Action::ActivateMana {
                card: guide,
                ability: 0,
                payment: [0, 0, 0, 1, 0, 0],
                counter_payment: Counters::default(),
                production: [1, 0, 0, 0, 0, 0],
            },
        );
        assert_eq!(child.state.players[p.index()].mana, [1, 0, 0, 0, 0, 0]);
        assert_eq!(g.snapshot(), before);
        child.restore(&before).unwrap();
        assert!(!child.can_fund(goal));
    }
}
#[test]
fn tagged_mana_funding_virtual_moon_departure_restores_intrinsic_and_granted_mana() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let forest = named(&mut g, p, "Forest", Zone::Battlefield);
        let horizons = named(&mut g, p, "New Horizons", Zone::Battlefield);
        g.state.objects[horizons as usize].attached_to = Some(g.object_ref(forest));
        let moon = named(&mut g, p, "Imprisoned in the Moon", Zone::Hand);
        cast(&mut g, moon, None, &[Target::Card(forest)]);
        settle(&mut g);
        g.state.players[p.index()].mana = [0; 6];
        let goal = FundingGoal {
            actor: p,
            required: [2, 0, 0, 0, 0, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        let mut virtual_world = FundingState::from_game(&g, &goal);
        assert!(!virtual_world.reachable(&goal, &mut HashSet::new()));
        virtual_world
            .resources
            .iter_mut()
            .find(|r| r.id == moon)
            .unwrap()
            .object
            .zone = Zone::Graveyard;
        assert!(virtual_world.reachable(&goal, &mut HashSet::new()));
        let before = g.snapshot();
        let mut child = {
            let parent = g.clone();
            parent.clone()
        };
        child.sacrifice_batch(&[(p, moon)]);
        child.state_based_actions();
        assert_eq!(
            CharacteristicWorld::granted_mana(&virtual_world, forest),
            CharacteristicWorld::granted_mana(&child, forest)
        );
        assert!(CharacteristicWorld::subtypes(&virtual_world, forest).has(Card::Forest, "Forest"));
        assert_eq!(
            CharacteristicWorld::kinds(&virtual_world, forest),
            child.current_kinds(forest)
        );
        assert!(
            child
                .legal_actions()
                .contains(&Action::TapMana { card: forest })
        );
        let grantor = child.object_ref(horizons);
        checked(
            &mut child,
            Action::TapGrantedMana {
                card: forest,
                grantor,
                production: [2, 0, 0, 0, 0, 0],
            },
        );
        assert_eq!(child.state.players[p.index()].mana, [2, 0, 0, 0, 0, 0]);
        assert_eq!(g.snapshot(), before);
    }
}
