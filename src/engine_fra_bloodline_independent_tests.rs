use super::*;

fn finish_bloodline(g: &mut Game) {
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
fn prepared_bloodline(g: &mut Game, p: Player) -> (CardId, ObjectRef) {
    let source = put(
        g,
        p,
        "Bloodline Recollector // Ancestral Craving",
        Zone::Hand,
    );
    g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
    act(g, Action::AnnounceCast { card: source });
    pay(g);
    finish_bloodline(g);
    for _ in 0..3 {
        let victim = put(g, p.other(), "Bear Cub", Zone::Battlefield);
        g.move_object(victim, Zone::Graveyard);
    }
    g.begin_step(Step::End);
    g.prepare_triggers();
    finish_bloodline(g);
    let copy = g
        .state
        .preparations
        .as_ref()
        .unwrap()
        .iter()
        .find(|row| row.permanent == g.object_ref(source))
        .unwrap()
        .spell_copy;
    (source, copy)
}

#[test]
fn ancestral_craving_waits_for_interrupted_draws_before_life_loss_and_replays() {
    for p in [Player::P0, Player::P1] {
        for target in [p, p.other()] {
            let mut g = scenario(p);
            let (_, copy) = prepared_bloodline(&mut g, p);
            put(&mut g, target, "Lorehold, the Historian", Zone::Battlefield);
            g.state.priority = p;
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
            let life = g.state.players[target.index()].life;
            let hand = g.hand_count(target);
            act(&mut g, Action::AnnounceExile { card: copy.card });
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(target),
                },
            );
            pay(&mut g);
            for _ in 0..48 {
                if matches!(g.state.pending, Pending::MiracleReveal { .. }) {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            assert!(
                matches!(g.state.pending, Pending::MiracleReveal { player } if player == target)
            );
            assert_eq!(g.hand_count(target), hand + 1);
            assert_eq!(g.state.players[target.index()].life, life);
            assert!(g.observe(target).miracle.is_some());
            assert!(g.observe(target.other()).miracle.is_none());
            let saved = g.snapshot();
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                act(&mut g, Action::DeclineMiracle);
                finish_bloodline(&mut g);
                assert_eq!(g.hand_count(target), hand + 3);
                assert_eq!(g.state.players[target.index()].life, life - 3);
            }
        }
    }
}

#[test]
fn bloodline_trigger_cannot_prepare_its_reentered_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(
            &mut g,
            p,
            "Bloodline Recollector // Ancestral Craving",
            Zone::Battlefield,
        );
        for _ in 0..3 {
            let creature = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            g.move_object(creature, Zone::Graveyard);
        }
        g.begin_step(Step::End);
        g.prepare_triggers();
        assert!(!g.state.pending_triggers.is_empty() || !g.state.stack.is_empty());
        g.move_object(source, Zone::Exile);
        g.move_object(source, Zone::Battlefield);
        finish_bloodline(&mut g);
        assert!(!g.is_prepared(g.object_ref(source)));
    }
}
