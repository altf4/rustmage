//! Standalone StartYourEngines requires no unrelated static provider to be indexed.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const START_ONLY: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::StartYourEngines],
    }],
    ..CardRules::EMPTY
};

#[test]
fn paid_entry_starts_speed_for_a_standalone_provider_without_a_trigger() {
    let _scope =
        fixture_rules::activate(&[(Card::from_name("Prophetic Prism").unwrap(), &START_ONLY)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Prophetic Prism", Zone::Hand);
        let mana = g.state.players[p.index()].mana.iter().sum::<u32>();
        cast(&mut g, source, None);
        assert_eq!(
            g.state.players[p.index()].speed,
            0,
            "the spell has not entered"
        );
        assert_eq!(
            g.state.players[p.index()].mana.iter().sum::<u32>(),
            mana - 2
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].speed, 1);
        assert_eq!(g.state.players[p.other().index()].speed, 0);
        assert!(g.state.static_sources.contains(&source));
        assert!(g.state.pending_triggers.is_empty());
        g.move_object(source, Zone::Graveyard);
        g.state_based_actions();
        assert_eq!(
            g.state.players[p.index()].speed,
            1,
            "speed persists after departure"
        );
    }
}

#[test]
fn start_only_uses_current_battlefield_controller_and_never_reduces_speed() {
    let _scope =
        fixture_rules::activate(&[(Card::from_name("Prophetic Prism").unwrap(), &START_ONLY)]);
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Prophetic Prism", Zone::Graveyard);
        g.state_based_actions();
        assert_eq!(g.state.players[p.index()].speed, 0);
        g.move_object(source, Zone::Battlefield);
        g.state_based_actions();
        assert_eq!(g.state.players[p.index()].speed, 1);
        g.state.players[p.index()].speed = 4;
        g.state.objects[source as usize].controller = p.other();
        g.state_based_actions();
        assert_eq!(g.state.players[p.index()].speed, 4);
        assert_eq!(g.state.players[p.other().index()].speed, 1);
    }
}
