//! Bard draw and token replacement obligations.
use super::*;

fn bard(g: &mut Game, p: Player) -> CardId {
    put(g, p, "Bard, King of Dale", Zone::Battlefield)
}
fn stock(g: &mut Game, p: Player, n: usize) {
    for _ in 0..n {
        put(g, p, "Forest", Zone::Library);
    }
}
#[test]
fn hob_bard_paid_keywords_draw_step_and_multiple_providers() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = put(&mut g, p, "Bard, King of Dale", Zone::Hand);
        g.state.players[p.index()].mana = [1, 1, 0, 0, 0, 4];
        cast(&mut g, b, None);
        settle(&mut g);
        assert_eq!(stats(&g, b), (3, 5));
        assert!(
            g.keywords(b)
                .contains(Keywords::REACH.union(Keywords::VIGILANCE))
        );
        stock(&mut g, p, 20);
        g.begin_step(Step::Draw);
        g.draw(p);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        g.draw(p);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
        bard(&mut g, p);
        g.draw(p);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 7);
        g.begin_step(Step::Draw);
        g.draw(p);
        assert_eq!(
            g.state.players[p.index()].cards_drawn_this_turn,
            8,
            "first draw of a new draw step is exempt"
        );
        g.draw(p);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 12);
        let before = g.state.players[p.other().index()].cards_drawn_this_turn;
        g.draw(p.other());
        assert_eq!(
            g.state.players[p.other().index()].cards_drawn_this_turn,
            before + 1
        );
    }
}
#[test]
fn hob_bard_miracle_defers_doubled_draws_privately_and_restores() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        bard(&mut g, p);
        put(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
        stock(&mut g, p, 8);
        g.draw_many(p, 2);
        assert!(matches!(g.state.pending, Pending::MiracleReveal { .. }));
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        assert!(g.observe(p).miracle.is_some());
        assert!(g.observe(p.other()).miracle.is_none());
        contracts::branch_contract(g.clone());
        act(&mut g, Action::DeclineMiracle);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 4);
    }
}
#[test]
fn hob_bard_empty_draw_and_current_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = bard(&mut g, p);
        for id in g.state.players[p.index()].library.clone() {
            g.move_object(id, Zone::Outside);
        }
        g.begin_step(Step::Draw);
        g.draw(p);
        assert!(g.state.players[p.index()].attempted_empty_draw);
        assert!(!g.state.players[p.index()].drew_in_own_draw_step);
        stock(&mut g, p, 4);
        g.draw(p);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        g.change_controller(g.object_ref(b), p.other());
        g.begin_step(Step::Main1);
        g.draw(p);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        let before = g.state.players[p.other().index()].cards_drawn_this_turn;
        g.draw(p.other());
        assert_eq!(
            g.state.players[p.other().index()].cards_drawn_this_turn,
            before + 2
        );
    }
}
#[test]
fn hob_bard_token_doubling_stacks_with_other_replacements() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        bard(&mut g, p);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let spell = put(&mut g, p, "Snarl Song", Zone::Hand);
        cast(&mut g, spell, None);
        for _ in 0..50 {
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
            {
                break;
            }
            if matches!(g.state.pending, Pending::Replacement { .. }) {
                contracts::branch_contract(g.clone());
            }
            choose(&mut g, |a| {
                matches!(
                    a,
                    Action::Pass | Action::PutTrigger { .. } | Action::ChooseReplacement { .. }
                )
            });
        }
        let count = g
            .state
            .objects
            .iter()
            .filter(|o| {
                o.zone == Zone::Battlefield && o.controller == p && o.card == Card::SosFractalToken
            })
            .count();
        assert_eq!(count, 8);
    }
}

#[test]
fn hob_bard_suppressed_abilities_stop_both_replacements() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = bard(&mut g, p);
        let aura = put(&mut g, p, "Witness Protection", Zone::Hand);
        cast(&mut g, aura, Some(Target::Card(b)));
        settle(&mut g);
        assert!(!g.printed_abilities_available(b));
        let before = g.state.players[p.index()].cards_drawn_this_turn;
        g.draw(p);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 1);
        g.move_object(aura, Zone::Graveyard);
        g.draw(p);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 3);
    }
}
#[test]
fn hob_bard_replaced_draw_packet_survives_source_leaving_but_later_draw_rechecks() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let b = bard(&mut g, p);
        put(&mut g, p, "Lorehold, the Historian", Zone::Battlefield);
        stock(&mut g, p, 8);
        g.draw_many(p, 2);
        assert!(matches!(g.state.pending, Pending::MiracleReveal { .. }));
        g.move_object(b, Zone::Exile);
        act(&mut g, Action::DeclineMiracle);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
    }
}
