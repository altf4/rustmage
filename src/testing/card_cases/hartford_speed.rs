//! Focused printed outcomes; shared engine invariants live in core tests.
//! Persistent speed, its inherent trigger, and max-speed source grants.
use super::big_helpers::{checked, next_turn, settle};
use super::*;

#[test]
fn perilous_snare_starts_persistent_speed_and_grants_its_activation_only_at_four() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let vehicle = put(&mut g, p, "Detention Chariot", Zone::Battlefield);
        let snare = put(&mut g, p, "Perilous Snare", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: snare });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(enemy).zone, Zone::Exile);
        assert_eq!(g.observe(p.other()).players[p.index()].speed, 1);
        assert!(!additional_costs::has_craft_activated_ability(&g, snare));
        assert!(!g.legal_actions().iter().any(
            |a| matches!(a, Action::AnnounceSourceGrant { source, .. } if source.card == snare)
        ));
        let mut countered = g.clone();
        countered.lose_life(p.other(), 1);
        countered.prepare_triggers();
        let trigger = countered.state.stack.last().unwrap().id;
        countered.counter_stack_item(trigger, false);
        settle(&mut countered);
        assert_eq!(countered.state.players[p.index()].speed, 1);
        countered.lose_life(p.other(), 1);
        settle(&mut countered);
        assert_eq!(
            countered.state.players[p.index()].speed,
            1,
            "countered trigger consumes the turn's trigger"
        );
        g.lose_life(p.other(), 1);
        assert_eq!(
            g.state.players[p.index()].speed,
            1,
            "increase uses the stack"
        );
        assert!(
            g.state
                .pending_triggers
                .iter()
                .any(|t| t.source_less_rule() == Some(SourceLessRule::Speed))
        );
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].speed, 2);
        g.lose_life(p.other(), 1);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].speed, 2, "once per turn");
        let mut departed = g.clone();
        departed.move_object(snare, Zone::Graveyard);
        settle(&mut departed);
        assert_eq!(departed.state.players[p.index()].speed, 2);
        assert_eq!(departed.object_view(enemy).zone, Zone::Battlefield);
        assert!(!additional_costs::has_craft_activated_ability(
            &departed, snare
        ));
        next_turn(&mut departed);
        next_turn(&mut departed);
        departed.lose_life(p.other(), 1);
        settle(&mut departed);
        assert_eq!(
            departed.state.players[p.index()].speed,
            3,
            "speed persists independently of its initiating permanent"
        );
        for speed in [3, 4] {
            next_turn(&mut g);
            next_turn(&mut g);
            assert_eq!(g.state.active, p);
            g.lose_life(p.other(), 1);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].speed, speed);
        }
        assert!(additional_costs::has_craft_activated_ability(&g, snare));
        // next_turn stops at the new upkeep; this activation needs a main phase.
        for _ in 0..20 {
            if g.state.step == Step::Main1 {
                break;
            }
            checked(&mut g, Action::Pass);
        }
        assert_eq!(g.state.step, Step::Main1);
        let source = g.object_ref(snare);
        let action = Action::AnnounceSourceGrant { source, ability: 0 };
        let mut off_turn = g.clone();
        off_turn.state.active = p.other();
        assert!(!off_turn.legal_actions().contains(&action));
        checked(&mut g, action);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(snare),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(vehicle),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.counter_count(vehicle, CounterKind::PlusOne), 1);
        assert!(g.object_view(snare).tapped);
        g.move_object(snare, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].speed, 4);
        assert!(
            !additional_costs::has_craft_activated_ability(&g, snare),
            "max-speed static grant functions on the battlefield"
        );
    }
}
