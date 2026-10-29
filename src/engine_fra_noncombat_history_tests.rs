//! Noncombat damage history and two gated Reality Fracture cards.
use super::*;

#[test]
fn noncombat_history_tracks_actual_player_damage_and_rolls_over() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let repriser = put(&mut g, p, "Grim Repriser", Zone::Graveyard);
        let action = Action::AnnounceActivate {
            card: repriser,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, action.clone());
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        settle(&mut g);
        assert!(g.observe(p).players[p.other().index()].noncombat_damage_this_turn);
        assert!(g.observe(p.other()).players[p.other().index()].noncombat_damage_this_turn);
        assert!(!g.observe(p).players[p.index()].noncombat_damage_this_turn);
        super::super::contracts::branch_contract(g.clone());
        act(&mut g, action);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(repriser).zone, Zone::Battlefield);
        assert_eq!(g.counter_count(repriser, CounterKind::Finality), 1);
        g.finish_cleanup();
        g.state.turn += 1;
        g.begin_turn();
        assert!(!g.observe(p).players[p.other().index()].noncombat_damage_this_turn);
        assert!(g.observe(p).players[p.other().index()].noncombat_damage_last_turn);
        g.finish_cleanup();
        g.state.turn += 1;
        g.begin_turn();
        assert!(!g.observe(p).players[p.other().index()].noncombat_damage_last_turn);
    }
}

#[test]
fn command_stage_counters_other_wizard_tokens_and_returns_next_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        g.create_tokens(p, Card(5298), 1, false);
        let existing = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.is_token()
                    && o.controller == p
                    && o.card == Card(5298))
                .then_some(id as CardId)
            })
            .unwrap();
        let spell = put(&mut g, p, "Command the Stage", Zone::Hand);
        cast(&mut g, spell, None);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        let tokens = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(id, o)| {
                (o.zone == Zone::Battlefield
                    && o.is_token()
                    && o.controller == p
                    && o.card == Card(5298))
                .then_some(id as CardId)
            })
            .collect::<Vec<_>>();
        assert_eq!(tokens.len(), 2);
        assert_eq!(g.counter_count(existing, CounterKind::PlusOne), 1);
        let new = *tokens.iter().find(|&&id| id != existing).unwrap();
        assert_eq!(g.counter_count(new, CounterKind::PlusOne), 0);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        settle(&mut g);
        g.finish_cleanup();
        g.state.turn += 1;
        g.begin_turn();
        g.begin_step(Step::Upkeep);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Hand);
    }
}

#[test]
fn noncombat_fact_distinguishes_player_combat_and_zero_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let repriser = put(&mut g, p, "Grim Repriser", Zone::Graveyard);
        let action = Action::AnnounceActivate {
            card: repriser,
            ability: 0,
            counter_payment: Counters::default(),
        };
        let event = |target, amount, combat| DamageEvent {
            preventable: true,
            source: g.damage_source(source),
            target,
            amount,
            combat,
        };
        let events = [
            event(Target::Player(p.other()), 0, false),
            event(Target::Player(p.other()), 1, true),
            event(Target::Card(source), 1, false),
            event(Target::Player(p), 1, false),
        ];
        g.apply_damage_batch(&events);
        assert!(g.observe(p).players[p.index()].noncombat_damage_this_turn);
        assert!(!g.observe(p).players[p.other().index()].noncombat_damage_this_turn);
        reject(&mut g, action);
    }
}
