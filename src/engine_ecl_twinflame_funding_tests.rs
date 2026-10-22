//! Frozen triggered mana uses the same owned and speculative characteristics.
use super::super::ability_timeline_tests::{checked, named, setup};
use super::super::ability_world::CharacteristicWorld;
use super::*;
fn all_types_boundary(g: &mut Game, source: CardId) {
    let timestamp = g.next_role_timestamp();
    let object = g.object_ref(source);
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            source: object,
            recipient: object,
            types: &[],
            additional: false,
            all_creature_types: true,
            expires_turn: None,
            timestamp,
        });
}
#[test]
fn ecl_twinflame_funding_virtual_provider_departure_removal_and_source_control_agree() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let provider = named(&mut g, p, "Twinflame Travelers", Zone::Battlefield);
        let source = named(&mut g, p, "Morcant's Eyes", Zone::Battlefield);
        all_types_boundary(&mut g, source);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 1,
            reserve: None,
            source: None,
        };
        let mut world = FundingState::from_game(&g, &goal);
        assert_eq!(world.trigger_occurrences(source, false), 2);
        world
            .resources
            .iter_mut()
            .find(|r| r.id == provider)
            .unwrap()
            .object
            .zone = Zone::Exile;
        assert_eq!(world.trigger_occurrences(source, false), 1);
        assert_eq!(g.trigger_occurrences(source, false), 2);
        let mut child = g.clone();
        child.move_object(provider, Zone::Exile);
        assert_eq!(
            world.trigger_occurrences(source, false),
            child.trigger_occurrences(source, false)
        );
        let mut world = FundingState::from_game(&g, &goal);
        world
            .resources
            .iter_mut()
            .find(|r| r.id == provider)
            .unwrap()
            .object
            .controller = p.other();
        assert_eq!(world.trigger_occurrences(source, false), 1);
        let mut child = g.clone();
        child.record_control(child.object_ref(provider), p.other(), None);
        assert_eq!(
            world.trigger_occurrences(source, false),
            child.trigger_occurrences(source, false)
        );
    }
}
#[test]
fn ecl_twinflame_triggered_mana_bonus_is_frozen_before_departure_and_funding_matches_actual() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let provider = named(&mut g, p, "Twinflame Travelers", Zone::Battlefield);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        let aura = super::super::ecl_green_colorless_tests::enter_growth(&mut g, p, land, 3);
        // Typed boundary fixture for an Elemental mana-trigger source: ordinary
        // Growth has no creature subtype. This isolates the shared mana hook
        // which Lavaleaper's printed Elemental ability will also consume.
        g.state.players[p.index()].mana = [0; 6];
        let ordinary = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 2,
            reserve: None,
            source: None,
        };
        assert!(FundingState::from_game(&g, &ordinary).reachable(&ordinary, &mut HashSet::new()));
        all_types_boundary(&mut g, aura);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 3,
            reserve: None,
            source: None,
        };
        g.state.players[p.index()].mana = [0; 6];
        let world = FundingState::from_game(&g, &goal);
        let captured = world.mana_tap_bonus(land, false);
        assert_eq!(captured, [0, 0, 0, 2, 0, 0]);
        assert_eq!(g.mana_tap_bonus(land), captured);
        assert!(
            world.reachable(&goal, &mut HashSet::new()),
            "upper={} optimistic={} moves={:?}",
            world.total_mana_upper(),
            world.optimistic(&goal),
            world
                .moves(&goal)
                .iter()
                .map(|m| (world.resources[m.resource].id, m.production, m.bonus))
                .collect::<Vec<_>>()
        );
        let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, p, "Goatnap", Zone::Hand);
        let saved = g.snapshot();
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        checked(&mut g, Action::TapMana { card: land });
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 2, 1, 0]);
        assert!(g.state.stack.is_empty() && g.state.pending_triggers.is_empty());
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 0, 2, 1, 0],
            },
        );
        super::super::ability_timeline_tests::settle(&mut g);
        assert_eq!(g.object_view(victim).controller, p);
        g.restore(&saved).unwrap();
        g.move_object(provider, Zone::Graveyard);
        assert_eq!(g.mana_tap_bonus(land), [0, 0, 0, 1, 0, 0]);
        // The original event's frozen result remains usable after its provider leaves.
        g.add_mana(p, captured);
        assert_eq!(g.state.players[p.index()].mana, captured);
    }
}
