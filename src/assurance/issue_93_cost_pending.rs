//! Loyalty costs must complete before the activation is committed.
use super::*;

#[test]
fn loyalty_activation_waits_for_counter_replacement_choice() {
    for player in [Player::P0, Player::P1] {
        for direct in [false, true] {
            let mut g = scenario(player);
            let walker = put(
                &mut g,
                player,
                "Liliana, Dreadhorde General",
                Zone::Battlefield,
            );
            g.add_counter(walker, CounterKind::Loyalty, 6);
            put(
                &mut g,
                player,
                "Doc Samson, Super Psychiatrist",
                Zone::Battlefield,
            );
            put(&mut g, player, "Doubling Season", Zone::Battlefield);
            put(&mut g, player, "Doubling Season", Zone::Battlefield);
            let observer = put(&mut g, player, "Way of the Paradox", Zone::Battlefield);
            if direct {
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| {
                        matches!(a,
                Action::Activate { card, ability: 0, .. } if *card == walker)
                    })
                    .unwrap();
                act(&mut g, action);
            } else {
                act(
                    &mut g,
                    Action::AnnounceActivate {
                        card: walker,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                pay(&mut g);
            }
            assert!(matches!(g.state.pending, Pending::Replacement { .. }));
            assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 6);
            assert!(!g.object_view(walker).loyalty_activated_this_turn);
            assert!(!g.state.players[player.index()].loyalty_ability_activated_this_turn);
            assert!(
                !g.state
                    .pending_triggers
                    .iter()
                    .any(|item| item.source.is_some_and(|s| s.id == observer))
            );
            let checkpoint = g.snapshot();
            while g
                .legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseReplacement { .. }))
            {
                choose(&mut g, |a| matches!(a, Action::ChooseReplacement { .. }));
            }
            assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 14);
            assert!(g.object_view(walker).loyalty_activated_this_turn);
            assert!(g.state.players[player.index()].loyalty_ability_activated_this_turn);
            let complete = g.snapshot();
            g.restore(&checkpoint).unwrap();
            while g
                .legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseReplacement { .. }))
            {
                choose(&mut g, |a| matches!(a, Action::ChooseReplacement { .. }));
            }
            assert_eq!(g.snapshot(), complete);
        }
    }
}

#[test]
fn creature_activation_observer_waits_for_counter_cost_replacements() {
    for player in [Player::P0, Player::P1] {
        for direct in [false, true] {
            let mut g = scenario(player);
            let source = put(
                &mut g,
                player,
                "Jessica Jones, Private Eye",
                Zone::Battlefield,
            );
            let observer = put(&mut g, player, "Elrond, Moon Reader", Zone::Battlefield);
            put(
                &mut g,
                player,
                "Doc Samson, Super Psychiatrist",
                Zone::Battlefield,
            );
            put(&mut g, player, "Loading Zone", Zone::Battlefield);
            if direct {
                let action = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| {
                        matches!(a,
                    Action::Activate { card, ability: 0, .. } if *card == source)
                    })
                    .unwrap();
                act(&mut g, action);
            } else {
                act(
                    &mut g,
                    Action::AnnounceActivate {
                        card: source,
                        ability: 0,
                        counter_payment: Counters::default(),
                    },
                );
                pay(&mut g);
            }
            assert!(matches!(g.state.pending, Pending::Replacement { .. }));
            assert!(g.object_view(source).tapped);
            assert_eq!(g.counter_count(source, CounterKind::Stun), 0);
            assert!(
                !g.state
                    .pending_triggers
                    .iter()
                    .any(|item| item.source.is_some_and(|s| s.id == observer))
            );
            let checkpoint = g.snapshot();
            while g
                .legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseReplacement { .. }))
            {
                choose(&mut g, |a| matches!(a, Action::ChooseReplacement { .. }));
            }
            assert!(g.counter_count(source, CounterKind::Stun) > 0);
            assert!(
                g.state
                    .pending_triggers
                    .iter()
                    .chain(&g.state.stack)
                    .chain(&g.state.ordering_triggers)
                    .any(|item| item.source.is_some_and(|s| s.id == observer))
            );
            let complete = g.snapshot();
            g.restore(&checkpoint).unwrap();
            while g
                .legal_actions()
                .iter()
                .any(|a| matches!(a, Action::ChooseReplacement { .. }))
            {
                choose(&mut g, |a| matches!(a, Action::ChooseReplacement { .. }));
            }
            assert_eq!(g.snapshot(), complete);
        }
    }
}
