use super::*;

#[test]
fn guiding_hydra_paid_x_requires_white_and_the_selected_generic_mana() {
    for player in [Player::P0, Player::P1] {
        for x in [1u32, 2] {
            let mut g = scenario(player);
            let hydra = put(&mut g, player, "Guiding Hydra", Zone::Hand);
            g.state.players[player.index()].mana = [1, 0, 0, 0, 0, x];
            act(&mut g, Action::AnnounceCast { card: hydra });
            reject(&mut g, Action::ChooseX { value: x + 1 });
            act(&mut g, Action::ChooseX { value: x });
            pay(&mut g);
            assert_eq!(g.state.players[player.index()].mana, [0; 6]);
            let saved = g.snapshot();
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                settle(&mut g);
                assert_eq!(g.object_view(hydra).zone, Zone::Battlefield);
                assert_eq!(g.counter_count(hydra, CounterKind::PlusOne), u64::from(x));
                assert_eq!(stats(&g, hydra), (1 + i128::from(x), i128::from(x)));
            }
        }
        let mut g = scenario(player);
        let hydra = put(&mut g, player, "Guiding Hydra", Zone::Hand);
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
        reject(&mut g, Action::AnnounceCast { card: hydra });
        assert_eq!(g.object_view(hydra).zone, Zone::Hand);
    }
}
