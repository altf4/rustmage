use super::*;

fn blocks(g: &mut Game, attacker: CardId, blocker: CardId) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(g, Action::Attack { card: attacker });
    act(g, Action::FinishAttackers);
    for _ in 0..16 {
        if g.state.pending == Pending::Blockers {
            break;
        }
        act(g, Action::Pass);
    }
    assert_eq!(g.state.pending, Pending::Blockers);
    act(g, Action::Block { blocker, attacker });
    act(g, Action::FinishBlockers);
}

#[test]
fn pursuer_blocker_last_controller_is_used_after_departure() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        put(
            &mut g,
            player,
            "Tetsuko Umezawa, Pursuer",
            Zone::Battlefield,
        );
        let attacker = put(&mut g, player, "Hill Giant", Zone::Battlefield);
        let blocker = put(&mut g, player.other(), "Llanowar Elves", Zone::Battlefield);
        let before = g.state.players[player.index()].life;
        blocks(&mut g, attacker, blocker);
        let source = g.object_ref(blocker);
        g.change_controller(source, player);
        g.move_object(blocker, Zone::Graveyard);
        let saved = g.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.state.players[player.index()].life, before - 1);
        }
    }
}

#[test]
fn copied_or_suppressed_pursuer_uses_current_trigger_abilities() {
    for player in [Player::P0, Player::P1] {
        for copied in [false, true] {
            let mut g = scenario(player);
            let source = put(
                &mut g,
                player,
                if copied {
                    "Bear Cub"
                } else {
                    "Tetsuko Umezawa, Pursuer"
                },
                Zone::Battlefield,
            );
            if copied {
                let recipe = CopyRecipe::new(Card::from_name("Tetsuko Umezawa, Pursuer").unwrap());
                g.apply_temporary_copy(g.object_ref(source), recipe, CopyDeadline::EndOfTurn);
            } else {
                let aura = put(
                    &mut g,
                    player.other(),
                    "Witness Protection",
                    Zone::Battlefield,
                );
                g.state.objects[aura as usize].attached_to = Some(g.object_ref(source));
            }
            let attacker = put(&mut g, player, "Hill Giant", Zone::Battlefield);
            let blocker = put(&mut g, player.other(), "Llanowar Elves", Zone::Battlefield);
            let before = g.state.players[player.other().index()].life;
            blocks(&mut g, attacker, blocker);
            settle(&mut g);
            assert_eq!(
                g.state.players[player.other().index()].life,
                before - i128::from(copied)
            );
        }
    }
}
