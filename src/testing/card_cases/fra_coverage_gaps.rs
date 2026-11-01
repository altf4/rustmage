//! Focused printed outcomes; shared engine invariants live in core tests.
//! Paid scenarios for FRA obligations not exercised by the bounded cast traces.
use super::*;

fn advance_until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("decision not reached: {:?}", g.state.pending);
}

fn finish(g: &mut Game) {
    advance_until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
    });
}

#[test]
fn ruric_two_prowess_triggers_and_opponent_target_draw() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let ruric = put(&mut g, player, "Ruric Thar, Biomagus", Zone::Battlefield);
        let spell = put(&mut g, player, "Shock", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(player.other())));
        g.prepare_triggers();
        let mut abilities: Vec<_> = g
            .state
            .ordering_triggers
            .iter()
            .filter_map(|item| {
                if item.source.is_some_and(|source| source.id == ruric)
                    && let StackKind::TriggeredAbility { ability } = item.kind
                {
                    return Some(ability);
                }
                None
            })
            .collect();
        abilities.sort_unstable();
        assert_eq!(abilities, vec![0, 1], "two independent prowess triggers");
        finish(&mut g);
        assert_eq!(stats(&g, ruric), (6, 8), "two separate prowess abilities");
        assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 0);
        g.finish_cleanup();
        assert_eq!(stats(&g, ruric), (4, 6));

        let opponent = player.other();
        g.state.players[opponent.index()].mana = [8; 6];
        let shock = put(&mut g, opponent, "Shock", Zone::Hand);
        g.state.priority = opponent;
        cast(&mut g, shock, Some(Target::Card(ruric)));
        finish(&mut g);
        assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 1);
        assert_eq!(g.object_view(ruric).damage, 2);
    }
}
