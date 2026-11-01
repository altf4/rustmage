use super::*;

fn finish_trigger(g: &mut Game) {
    for _ in 0..48 {
        if g.outcome().is_some()
            || (g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none())
        {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::DeclineMiracle
            )
        });
    }
    panic!("Fblthp trigger did not finish");
}

#[test]
fn fblthp_wins_on_zero_or_one_card_even_when_draw_attempt_fails() {
    for player in [Player::P0, Player::P1] {
        for count in [0, 1] {
            let mut g = scenario(player);
            let fblthp = put(&mut g, player, "Fblthp, Impossibly Lost", Zone::Battlefield);
            let dealer = put(&mut g, player, "Bear Cub", Zone::Battlefield);
            put(&mut g, player, "Lorehold, the Historian", Zone::Battlefield);
            for card in g.state.players[player.index()].library.clone() {
                g.move_object(card, Zone::Outside);
            }
            for _ in 0..count {
                put(&mut g, player, "Forest", Zone::Library);
            }
            let event = DamageEvent {
                preventable: true,
                source: g.damage_source(dealer),
                target: Target::Player(player.other()),
                amount: 1,
                combat: true,
            };
            g.apply_damage_batch(&[event]);
            let saved = g.snapshot();
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                finish_trigger(&mut g);
                assert_eq!(g.outcome(), Some(Outcome::Winner(player)));
                assert_eq!(g.object_view(fblthp).zone, Zone::Battlefield);
                assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, count);
                assert!(g.state.players[player.index()].attempted_empty_draw);
            }
        }
    }
}

#[test]
fn fblthp_trigger_controller_draws_and_owner_shuffles_after_control_change() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let fblthp = put(
            &mut g,
            player.other(),
            "Fblthp, Impossibly Lost",
            Zone::Battlefield,
        );
        let reference = g.object_ref(fblthp);
        g.change_controller(reference, player);
        let dealer = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let owner_library = g.state.players[player.other().index()].library.len();
        let library = g.state.players[player.index()].library.len();
        let event = DamageEvent {
            preventable: true,
            source: g.damage_source(dealer),
            target: Target::Player(player.other()),
            amount: 1,
            combat: true,
        };
        g.apply_damage_batch(&[event]);
        g.change_controller(reference, player.other());
        let saved = g.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            finish_trigger(&mut g);
            assert_eq!(g.state.players[player.index()].library.len(), library - 2);
            assert_eq!(
                g.state.players[player.other().index()].library.len(),
                owner_library + 1
            );
            assert_eq!(g.object_view(fblthp).zone, Zone::Library);
            assert_eq!(g.object_view(fblthp).owner, player.other());
        }
    }
}
