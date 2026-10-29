use super::*;

#[test]
fn anguish_actual_cast_requires_red_and_pays_chosen_x_with_rollback() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let card = put(&mut g, player, "Ajani's Anguish", Zone::Hand);
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
        let missing_red = g.snapshot();
        reject(&mut g, Action::AnnounceCast { card });
        assert_eq!(g.snapshot(), missing_red);
        assert_eq!(g.object_view(card).zone, Zone::Hand);

        g.state.players[player.index()].mana = [0, 0, 0, 1, 0, 2];
        let initial = g.snapshot();
        let mut completed = None;
        for _ in 0..2 {
            act(&mut g, Action::AnnounceCast { card });
            let before_invalid_x = g.snapshot();
            reject(&mut g, Action::ChooseX { value: 3 });
            assert_eq!(g.snapshot(), before_invalid_x);
            act(&mut g, Action::ChooseX { value: 2 });
            pay(&mut g);
            assert_eq!(g.state.players[player.index()].mana, [0; 6]);
            assert_eq!(g.state.stack.last().unwrap().x, Some(2));
            let before_life = g.state.players[player.other().index()].life;
            let mut targeted = false;
            for _ in 0..96 {
                g.prepare_triggers();
                let actions = g.legal_actions();
                let target = Action::ChooseTarget { target: Target::Player(player.other()) };
                if !targeted && actions.contains(&target) {
                    act(&mut g, target);
                    targeted = true;
                } else if actions.contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                } else if targeted && g.state.pending == Pending::Priority
                    && g.state.stack.is_empty() && g.state.pending_triggers.is_empty()
                    && g.state.resolving.is_none() { break; }
                else if let Some(a) = actions.into_iter().find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. })) {
                    act(&mut g, a);
                } else { panic!("unexpected Anguish entry state {:?}", g.state.pending); }
            }
            assert!(targeted);
            assert_eq!(g.object_view(card).zone, Zone::Battlefield);
            assert_eq!(g.state.players[player.other().index()].life, before_life - 2);
            let after = g.snapshot();
            if let Some(previous) = &completed { assert_eq!(&after, previous); }
            completed = Some(after);
            g.restore(&initial).unwrap();
            assert_eq!(g.snapshot(), initial);
        }
    }
}
