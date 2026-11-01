use super::*;
use super::*;
use crate::cards::{AnimationSpec, ReplacementFilter, ReplacementSpec};
pub use rustmage_core::engine::replacements::*;
use std::collections::VecDeque;
#[cfg(test)]
mod entering_power_filter_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{named, setup};

    #[test]
    fn ordinary_entry_power_preview_fast_path() {
        let mut game = setup(Player::P0);
        let creature = named(&mut game, Player::P0, "Bear Cub", Zone::Hand);
        named(&mut game, Player::P0, "Doubling Season", Zone::Battlefield);
        let card = game.state.objects[creature as usize].card;
        let start = std::time::Instant::now();
        for _ in 0..10_000 {
            assert!(!std::hint::black_box(entry_power_sensitive(
                &game,
                Player::P0,
                card
            )));
        }
        eprintln!(
            "ordinary entry power-filter checks: 10000 in {:?}",
            start.elapsed()
        );
    }

    #[test]
    fn incoming_power_predicate_rechecks_after_counter_replacement() {
        for player in [Player::P0, Player::P1] {
            let mut game = setup(player);
            let vacuum = named(&mut game, player, "Ghost Vacuum", Zone::Graveyard);
            let card = named(&mut game, player, "Hill Giant", Zone::Exile);
            let mut movement =
                entries::EntryMove::from_view(game.object_view(card), Zone::Battlefield, false);
            movement.controller = player;
            movement.ghost_vacuum_after = Some(game.object_ref(vacuum));
            movement.counters.add(CounterKind::PlusOne, 1);
            let mut placement = CounterPlacement {
                object: movement.before.reference,
                placed_by: player,
                card: movement.before.card,
                owner: movement.before.owner,
                controller: player,
                protector: movement.battle_protector,
                subtypes: Game::entry_choice_subtypes(&movement),
                kind: movement.entry_kinds(),
                power: game.ghost_entry_preview(&movement).unwrap().power,
                amount: movement.counters.clone(),
                effect: true,
                entry_index: Some(0),
                private: false,
            };
            let limited = ReplacementSpec::Counters {
                filter: ReplacementFilter::CreaturePowerAtMost(2),
                kind: Some(CounterKind::PlusOne),
                factor: 1,
                add: 1,
                effects_only: false,
            };
            assert_eq!(placement.power, 2);
            assert!(counter_applies(limited, &placement));
            modify_counter(limited, &mut placement);
            movement.counters = placement.amount.clone();
            let changed = game.ghost_entry_preview(&movement).unwrap();
            placement.power = changed.power;
            placement.kind = changed.kinds;
            placement.subtypes = changed.granted_subtypes;
            assert_eq!(placement.power, 3);
            assert!(!counter_applies(limited, &placement));
        }
    }
}
#[cfg(test)]
mod blight_cost_zero_replacement_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{checked, named, settle, setup};

    #[test]
    fn replacement_to_zero_still_pays_declared_blight_cost() {
        for player in [Player::P0, Player::P1] {
            let mut g = setup(player);
            let spell = named(&mut g, player, "Pyrrhic Strike", Zone::Hand);
            let artifact = named(&mut g, player.other(), "Short Bow", Zone::Battlefield);
            let victim = named(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
            let cost_creature = named(&mut g, player, "Bear Cub", Zone::Battlefield);
            named(
                &mut g,
                player,
                "Doc Samson, Super Psychiatrist",
                Zone::Battlefield,
            );
            checked(&mut g, Action::AnnounceCast { card: spell });
            checked(&mut g, Action::ChooseBlight { pay: true });
            checked(&mut g, Action::ChooseMode { mode: 2 });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(artifact),
                },
            );
            if !g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(victim),
            }) {
                checked(&mut g, Action::FinishTargets);
            }
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            checked(&mut g, Action::BeginOrderedCosts);
            let blight = g
                .state
                .announcing
                .as_ref()
                .unwrap()
                .cast_costs
                .as_ref()
                .unwrap()
                .clauses
                .iter()
                .position(|r| matches!(r.clause, CastCostClause::Blight { .. }))
                .unwrap() as u16;
            checked(&mut g, Action::ChooseCastCost { clause: blight });
            let subject = g.object_ref(cost_creature);
            // Enter the real cost continuation and model a replacement that turns
            // the proposed placement into zero, a legal modified payment (CR118.11).
            g.pay_blight(subject);
            let state = g.state.replacements.as_mut().unwrap();
            let Some(Work::Counters(batch)) = state.work.front_mut() else {
                panic!("counter payment staged")
            };
            batch.placements[0].amount = Counters::default();
            assert!(!g.advance_replacements(true));
            assert_eq!(g.counter_count(cost_creature, CounterKind::MinusOne), 0);
            let mana = g
                .state
                .announcing
                .as_ref()
                .unwrap()
                .cast_costs
                .as_ref()
                .unwrap()
                .clauses
                .iter()
                .position(|r| r.clause == CastCostClause::Mana)
                .unwrap() as u16;
            assert!(
                g.state
                    .announcing
                    .as_ref()
                    .unwrap()
                    .cast_costs
                    .as_ref()
                    .unwrap()
                    .clauses[usize::from(blight)]
                .paid
            );
            checked(&mut g, Action::ChooseCastCost { clause: mana });
            let payment = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, payment);
            settle(&mut g);
            assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
            assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        }
    }
}
#[cfg(test)]
mod proliferate_player_replacement_unit_tests {
    use super::*;
    use crate::engine::assurance::big_helpers::{named, setup};

    #[test]
    fn typed_replacement_amounts_compose_in_chosen_order() {
        let double = ReplacementSpec::PlayerCounters {
            kind: crate::cards::PlayerCounterKind::Poison,
            relation: CastRelation::Any,
            placed_by: None,
            effects_only: false,
            factor: 2,
            add: 0,
        };
        let plus = ReplacementSpec::PlayerCounters {
            kind: crate::cards::PlayerCounterKind::Poison,
            relation: CastRelation::Any,
            placed_by: None,
            effects_only: false,
            factor: 1,
            add: 1,
        };
        assert_eq!(
            modified_player_counters(plus, modified_player_counters(double, 1)),
            3
        );
        assert_eq!(
            modified_player_counters(double, modified_player_counters(plus, 1)),
            4
        );
        let game = setup(Player::P0);
        let source = game.object_ref(0);
        let mut event = ProliferateBatch {
            player: Player::P0,
            amount: 1,
            used: Vec::new(),
        };
        let sibling = modify_proliferate(
            ReplacementSpec::Proliferate {
                relation: CastRelation::You,
                factor: 2,
                add: 0,
            },
            &mut event,
            source,
            0,
        )
        .unwrap();
        assert_eq!(event.amount, 1);
        assert_eq!(sibling.count, 1);
        assert_eq!(sibling.used, vec![(source, 0)]);
        let suppressed = modify_proliferate(
            ReplacementSpec::Proliferate {
                relation: CastRelation::You,
                factor: 0,
                add: 0,
            },
            &mut event,
            source,
            1,
        );
        assert!(suppressed.is_none());
        assert_eq!(event.amount, 0);
        // Cancelling one child cannot mutate the separately retained sibling.
        assert_eq!(sibling.used, vec![(source, 0)]);
    }

    #[test]
    fn player_replacement_distinguishes_recipient_placer_and_cost() {
        let effect_only = ReplacementSpec::PlayerCounters {
            kind: crate::cards::PlayerCounterKind::Poison,
            relation: CastRelation::Opponent,
            placed_by: Some(CastRelation::You),
            effects_only: true,
            factor: 2,
            add: 0,
        };
        assert!(player_counter_rule_matches(
            effect_only,
            Player::P0,
            Player::P1,
            Player::P0,
            true,
            crate::cards::PlayerCounterKind::Poison
        ));
        assert!(!player_counter_rule_matches(
            effect_only,
            Player::P0,
            Player::P0,
            Player::P0,
            true,
            crate::cards::PlayerCounterKind::Poison
        ));
        assert!(!player_counter_rule_matches(
            effect_only,
            Player::P0,
            Player::P1,
            Player::P1,
            true,
            crate::cards::PlayerCounterKind::Poison
        ));
        assert!(!player_counter_rule_matches(
            effect_only,
            Player::P0,
            Player::P1,
            Player::P0,
            false,
            crate::cards::PlayerCounterKind::Poison
        ));
        let cost_and_effect = ReplacementSpec::PlayerCounters {
            kind: crate::cards::PlayerCounterKind::Poison,
            relation: CastRelation::Opponent,
            placed_by: Some(CastRelation::You),
            effects_only: false,
            factor: 0,
            add: 0,
        };
        assert!(player_counter_rule_matches(
            cost_and_effect,
            Player::P0,
            Player::P1,
            Player::P0,
            false,
            crate::cards::PlayerCounterKind::Poison
        ));
        assert_eq!(modified_player_counters(cost_and_effect, 5), 0);
        // A zero-result replacement does not turn the original cost into an
        // effects-only event for another player-counter replacement.
        assert!(!player_counter_rule_matches(
            effect_only,
            Player::P0,
            Player::P1,
            Player::P0,
            false,
            crate::cards::PlayerCounterKind::Poison
        ));
    }

    #[test]
    fn simultaneous_counter_choices_interleave_affected_players_in_turn_order() {
        for active in [Player::P0, Player::P1] {
            let mut game = setup(active);
            let id = named(&mut game, active.other(), "Bear Cub", Zone::Battlefield);
            let object = game.object_ref(id);
            let creature = &game.state.objects[id as usize];
            let placement = CounterPlacement {
                object,
                placed_by: active,
                card: creature.card,
                owner: creature.owner,
                controller: creature.controller,
                protector: None,
                subtypes: game.granted_subtypes(id),
                kind: game.current_kinds(id),
                power: game.stats(id).power,
                amount: Counters::of(CounterKind::PlusOne, 1),
                effect: true,
                entry_index: None,
                private: false,
            };
            let mut player_events = player_counter_events::PlayerCounterEvents::default();
            player_events.push(active, crate::cards::PlayerCounterKind::Poison, 1, 0);
            player_events.push(
                active.other(),
                crate::cards::PlayerCounterKind::Poison,
                1,
                0,
            );
            let mut batch = CounterBatch {
                player_events,
                self_effects: Vec::new(),
                player_placed_by: Some(active),
                player_effect: [true; 2],
                next_player: 0,
                proliferated: Some(active),
                completion: None,
                placements: vec![placement],
                next: 0,
                used: Vec::new(),
            };
            assert_eq!(
                current_player_counter(&batch, active),
                Some((active, crate::cards::PlayerCounterKind::Poison))
            );
            batch.next_player = 1;
            assert_eq!(current_player_counter(&batch, active), None);
            batch.next = 1;
            assert_eq!(
                current_player_counter(&batch, active),
                Some((active.other(), crate::cards::PlayerCounterKind::Poison))
            );
            batch.next_player = 2;
            assert_eq!(current_player_counter(&batch, active), None);
        }
    }
}
