use super::*;

fn begin_blocks(g: &mut Game, attacker: CardId) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card: attacker });
    act(g, Action::FinishAttackers);
    for _ in 0..16 {
        if g.state.pending == Pending::Blockers { break; }
        act(g, Action::Pass);
    }
    assert_eq!(g.state.pending, Pending::Blockers);
}

fn order_block_triggers(g: &mut Game) {
    g.prepare_triggers();
    while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
        choose(g, |a| matches!(a, Action::PutTrigger { .. }));
    }
}

#[test]
fn tetsuko_observes_each_opposing_small_blocker_and_preserves_departed_blocker_lki() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let tetsuko = put(&mut g, p, "Tetsuko Umezawa, Pursuer", Zone::Battlefield);
        let attacker = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let elf = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let short = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let normal = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[short as usize].growth.toughness = -1;
        let life = g.state.players[p.other().index()].life;
        begin_blocks(&mut g, attacker);
        for blocker in [elf, short, normal] { act(&mut g, Action::Block { blocker, attacker }); }
        act(&mut g, Action::FinishBlockers);
        g.move_object(elf, Zone::Graveyard);
        g.move_object(tetsuko, Zone::Graveyard);
        order_block_triggers(&mut g);
        let saved = g.snapshot();
        super::super::contracts::branch_contract(g.clone());
        for replay in 0..2 {
            if replay == 1 { g.restore(&saved).unwrap(); }
            settle(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, life - 2);
        }
    }
}

#[test]
fn tetsuko_block_trigger_uses_blockers_live_controller_at_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Tetsuko Umezawa, Pursuer", Zone::Battlefield);
        let attacker = put(&mut g, p, "Hill Giant", Zone::Battlefield);
        let blocker = put(&mut g, p.other(), "Llanowar Elves", Zone::Battlefield);
        let own_life = g.state.players[p.index()].life;
        let enemy_life = g.state.players[p.other().index()].life;
        begin_blocks(&mut g, attacker);
        act(&mut g, Action::Block { blocker, attacker });
        act(&mut g, Action::FinishBlockers);
        g.change_controller(g.object_ref(blocker), p);
        order_block_triggers(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, own_life - 1);
        assert_eq!(g.state.players[p.other().index()].life, enemy_life);
    }
}

#[test]
fn tetsuko_prowess_and_double_strike_apply_to_actual_noncreature_cast() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let tetsuko = put(&mut g, p, "Tetsuko Umezawa, Pursuer", Zone::Battlefield);
        assert!(g.keywords(tetsuko).contains(Keywords::DOUBLE_STRIKE));
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, spell, Some(Target::Player(p.other())));
        settle(&mut g);
        assert_eq!(stats(&g, tetsuko), (3, 5));
        g.finish_cleanup();
        assert_eq!(stats(&g, tetsuko), (2, 4));
    }
}
