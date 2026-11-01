//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn bloodline_finish(g: &mut Game) {
    for _ in 0..96 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Bloodline resolution did not finish");
}
fn bloodline_enter(g: &mut Game, p: Player) -> CardId {
    let source = put(
        g,
        p,
        "Bloodline Recollector // Ancestral Craving",
        Zone::Hand,
    );
    g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
    act(g, Action::AnnounceCast { card: source });
    pay(g);
    bloodline_finish(g);
    assert!(!g.is_prepared(g.object_ref(source)));
    source
}
fn bloodline_deaths(g: &mut Game, p: Player, count: usize) {
    for n in 0..count {
        let owner = if n % 2 == 0 { p } else { p.other() };
        let creature = put(g, owner, "Bear Cub", Zone::Battlefield);
        g.move_object(creature, Zone::Graveyard);
    }
}

#[test]
fn bloodline_observes_all_players_deaths_and_rechecks_at_any_end_step() {
    {
        let p = Player::P0;
        for count in [2, 3] {
            let mut g = scenario(p);
            let source = bloodline_enter(&mut g, p);
            bloodline_deaths(&mut g, p, count);
            assert_eq!(g.state.creatures_died_this_turn, count as u64);
            g.state.active = p.other();
            g.begin_step(Step::End);
            g.prepare_triggers();

            bloodline_finish(&mut g);
            assert_eq!(g.is_prepared(g.object_ref(source)), count == 3);
            if count == 3 {
                let reference = g.object_ref(source);
                let copies = g
                    .state
                    .preparations
                    .as_ref()
                    .unwrap()
                    .iter()
                    .filter(|r| r.permanent == reference)
                    .count();
                assert_eq!(copies, 1);
                g.begin_step(Step::End);
                bloodline_finish(&mut g);
                assert_eq!(
                    g.state
                        .preparations
                        .as_ref()
                        .unwrap()
                        .iter()
                        .filter(|r| r.permanent == reference)
                        .count(),
                    1
                );
            }
        }
    }
}

#[test]
fn bloodline_deaths_replaced_by_exile_do_not_count() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = bloodline_enter(&mut g, p);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        bloodline_deaths(&mut g, p, 3);
        assert_eq!(g.state.creatures_died_this_turn, 0);
        g.begin_step(Step::End);
        bloodline_finish(&mut g);
        assert!(!g.is_prepared(g.object_ref(source)));
    }
}
