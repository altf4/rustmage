//! Triggered mana uses the same production-time owned and speculative characteristics.
use crate::ecl_scenarios::{checked, named, setup};
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use std::collections::HashSet;

fn occurrences<W: CharacteristicWorld + ?Sized>(world: &W, source: CardId) -> usize {
    1 + trigger_multiplicity::additional_triggers(
        &characteristic_query::CharacteristicQuery::in_world(world),
        source,
    )
}

#[test]
fn ecl_twinflame_funding_virtual_provider_departure_removal_and_source_control_agree() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let provider = named(&mut g, p, "Twinflame Travelers", Zone::Battlefield);
        let source = named(&mut g, p, "Lavaleaper", Zone::Battlefield);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 1,
            reserve: None,
            source: None,
        };
        let mut world = FundingState::from_game_with_convoke(&g, &goal, true);
        assert_eq!(occurrences(&world, source), 2);
        world
            .resources
            .iter_mut()
            .find(|r| r.id == provider)
            .unwrap()
            .object
            .zone = Zone::Exile;
        assert_eq!(occurrences(&world, source), 1);
        assert_eq!(occurrences(&g, source), 2);
        let mut child = g.clone();
        child.move_object(provider, Zone::Exile);
        assert_eq!(occurrences(&world, source), occurrences(&child, source));
        let mut world = FundingState::from_game_with_convoke(&g, &goal, true);
        world
            .resources
            .iter_mut()
            .find(|r| r.id == provider)
            .unwrap()
            .object
            .controller = p.other();
        assert_eq!(occurrences(&world, source), 1);
        let mut child = g.clone();
        child.record_control(child.object_ref(provider), p.other(), None);
        assert_eq!(occurrences(&world, source), occurrences(&child, source));
    }
}
#[test]
fn ecl_twinflame_triggered_mana_bonus_at_production_and_funding_match_actual() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        named(&mut g, p, "Lavaleaper", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        let ordinary = FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, 2, 0],
            generic: 0,
            reserve: None,
            source: None,
        };
        assert!(FundingState::from_game(&g, &ordinary).reachable(&ordinary, &mut HashSet::new()));
        let goal = FundingGoal {
            generic: 1,
            ..ordinary
        };
        assert!(!FundingState::from_game(&g, &goal).reachable(&goal, &mut HashSet::new()));
        let provider = named(&mut g, p, "Twinflame Travelers", Zone::Battlefield);
        let world = FundingState::from_game(&g, &goal);
        let tapped_land = g.object_ref(land);
        let tap = g.mana_tap_bonuses(p, land);
        let production = [0, 0, 0, 0, 1, 0];
        let produced_bonus = tap.with_trigger_bonuses(&g, p, Some(tapped_land));
        assert_eq!(
            produced_bonus.singleton(production),
            Some([0, 0, 0, 0, 2, 0])
        );
        let step = world
            .moves(&goal)
            .into_iter()
            .find(|step| world.resources[step.resource].id == land)
            .unwrap();
        let mut projected = None;
        world.visit_applied(&step, &mut |next| {
            projected = Some(next.mana);
            true
        });
        assert_eq!(projected, Some([0, 0, 0, 0, 3, 0]));
        assert!(
            world.reachable(&goal, &mut HashSet::new()),
            "upper={} optimistic={} moves={:?}",
            world.total_mana_upper(),
            world.optimistic(&goal),
            world
                .moves(&goal)
                .iter()
                .map(|m| (world.resources[m.resource].id, m.production))
                .collect::<Vec<_>>()
        );
        let spell = named(&mut g, p, "Thrashing Brontodon", Zone::Hand);
        let saved = g.snapshot();
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::TapMana { card: land });
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 3, 0]);
        assert!(g.state.stack.is_empty() && g.state.pending_triggers.is_empty());
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 3, 0],
            },
        );
        crate::ecl_scenarios::settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
        g.restore(&saved).unwrap();
        g.move_object(provider, Zone::Graveyard);
        assert_eq!(
            tap.with_trigger_bonuses(&g, p, Some(tapped_land))
                .singleton(production),
            Some(production)
        );
        assert!(!FundingState::from_game(&g, &goal).reachable(&goal, &mut HashSet::new()));
        // A new production event sees the provider's departure in both worlds.
        checked(&mut g, Action::TapMana { card: land });
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 2, 0]);
    }
}
