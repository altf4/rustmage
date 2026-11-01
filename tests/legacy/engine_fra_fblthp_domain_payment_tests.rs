use super::*;

#[test]
fn fblthp_requires_two_green_and_actual_x_mana() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let card = put(&mut g, player, "Fblthp, Knows the Way", Zone::Hand);
        g.state.players[player.index()].mana = [0, 0, 0, 0, 1, 2];
        reject(&mut g, Action::AnnounceCast { card });
        assert_eq!(g.object_view(card).zone, Zone::Hand);
        g.state.players[player.index()].mana = [0, 0, 0, 0, 2, 2];
        act(&mut g, Action::AnnounceCast { card });
        reject(&mut g, Action::ChooseX { value: 3 });
        act(&mut g, Action::ChooseX { value: 2 });
        pay(&mut g);
        assert_eq!(g.state.players[player.index()].mana, [0; 6]);
        let saved = g.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            for _ in 0..48 {
                if matches!(g.state.pending, Pending::EffectSearch { .. }) {
                    break;
                }
                g.prepare_triggers();
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            assert!(matches!(g.state.pending, Pending::EffectSearch { .. }));
            assert!(g.observe(player.other()).search_choice.is_none());
            act(&mut g, Action::FinishSearch);
            settle(&mut g);
            assert_eq!(g.object_view(card).zone, Zone::Battlefield);
        }
    }
}
