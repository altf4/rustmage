//! Borrowed entering world observes Ghost Vacuum's simultaneous 1/1 effect.
use super::*;
use crate::engine::assurance::big_helpers::{named, setup};

#[test]
fn projected_ghost_entry_layers_apply_counters_and_existing_static_bonus() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let vacuum = named(&mut g, p, "Ghost Vacuum", Zone::Graveyard);
        let creature = named(&mut g, p, "Hill Giant", Zone::Exile);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let mut movement =
            entries::EntryMove::from_view(g.object_view(creature), Zone::Battlefield, false);
        movement.controller = p;
        movement.ghost_vacuum_after = Some(g.object_ref(vacuum));
        let saved = g.snapshot();
        let world = FundingState::from_game(&g, &goal);
        let stats = world.preview_entry_stats(&movement).unwrap();
        assert_eq!((stats.power, stats.toughness), (1, 1));
        assert_eq!(g.snapshot(), saved);
        movement.counters.add(CounterKind::PlusOne, 1);
        let stats = world.preview_entry_stats(&movement).unwrap();
        assert_eq!((stats.power, stats.toughness), (2, 2));
        named(&mut g, p, "Anthem of Champions", Zone::Battlefield);
        let world = FundingState::from_game(&g, &goal);
        let stats = world.preview_entry_stats(&movement).unwrap();
        assert_eq!((stats.power, stats.toughness), (3, 3));
    }
}

#[test]
fn projected_ghost_entries_share_noncopy_effect_timestamp_and_flying_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let vacuum = named(&mut g, p, "Ghost Vacuum", Zone::Graveyard);
        let a = named(&mut g, p, "Bear Cub", Zone::Exile);
        let b = named(&mut g, p, "Hill Giant", Zone::Exile);
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let mut moves = Vec::new();
        for id in [a, b] {
            let mut m = entries::EntryMove::from_view(g.object_view(id), Zone::Battlefield, false);
            m.controller = p;
            m.ghost_vacuum_after = Some(g.object_ref(vacuum));
            m.counters.add(CounterKind::Flying, 1);
            moves.push(m);
        }
        let before = g.snapshot();
        let mut world = FundingState::from_game(&g, &goal);
        for id in [a, b] {
            if !world.resources.iter().any(|r| r.id == id) {
                world.resources.push(Resource {
                    id,
                    attacking: false,
                    object: g.state.objects[id as usize].clone(),
                });
            }
        }
        let mut checked = false;
        world.commit_projected_entries(moves, &[0, 1], &mut |next| {
            let query = characteristic_query::CharacteristicQuery::in_world(next);
            for id in [a, b] {
                let view = query.object_view(id);
                assert_eq!((view.power, view.toughness), (1, 1));
                assert!(view.has_subtype("Spirit"));
                assert!(query.keywords(id).contains(Keywords::FLYING));
            }
            let roles = next.entry_projection.as_ref().unwrap();
            let rows = &roles.roles.base_stats;
            let x = rows.iter().find(|r| r.recipient.card == a).unwrap();
            let y = rows.iter().find(|r| r.recipient.card == b).unwrap();
            assert_eq!(x.timestamp, y.timestamp);
            checked = true;
            true
        });
        assert!(checked);
        assert_eq!(g.snapshot(), before);
    }
}

#[test]
fn projected_ghost_replacement_chain_recomputes_power_for_each_counter_amount() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let vacuum = named(&mut g, p, "Ghost Vacuum", Zone::Graveyard);
        let creature = named(&mut g, p, "Hill Giant", Zone::Exile);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        named(
            &mut g,
            p,
            "Doc Samson, Super Psychiatrist",
            Zone::Battlefield,
        );
        let goal = FundingGoal {
            actor: p,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let mut movement =
            entries::EntryMove::from_view(g.object_view(creature), Zone::Battlefield, false);
        movement.controller = p;
        movement.ghost_vacuum_after = Some(g.object_ref(vacuum));
        movement.counters.add(CounterKind::PlusOne, 1);
        let world = FundingState::from_game(&g, &goal);
        let placement = replacements::CounterPlacement {
            object: movement.before.reference,
            placed_by: p,
            card: movement.before.card,
            owner: movement.before.owner,
            controller: p,
            protector: movement.battle_protector,
            subtypes: Game::entry_choice_subtypes(&movement),
            kind: movement.entry_kinds(),
            power: 2,
            amount: movement.counters.clone(),
            effect: true,
            entry_index: Some(0),
            private: false,
        };
        let powers = std::cell::RefCell::new(Vec::new());
        let refresh = |event: &replacements::CounterPlacement| {
            let mut next = event.clone();
            let mut candidate = movement.clone();
            candidate.counters = event.amount.clone();
            let view = world.preview_entry_view(&candidate).unwrap();
            powers.borrow_mut().push(view.power);
            next.kind = view.kinds;
            next.power = view.power;
            next.subtypes = view.granted_subtypes;
            next
        };
        replacements::counter_outcomes(&world, &placement, &mut Vec::new(), &refresh, &mut |_| {
            false
        });
        let powers = powers.into_inner();
        assert!(powers.contains(&2));
        assert!(powers.contains(&3));
        assert!(powers.contains(&4));
        assert!(powers.contains(&5));
    }
}
