use super::*;

#[test]
fn witchstalker_can_return_preexisting_graveyard_land_after_full_mill() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let old_land = put(&mut g, player, "Forest", Zone::Graveyard);
        let wolf = put(&mut g, player, "Primal Witchstalker", Zone::Hand);
        cast(&mut g, wolf, None);
        let mut selected = false;
        for _ in 0..48 {
            g.prepare_triggers();
            let actions = g.legal_actions();
            if actions.contains(&Action::ChooseTarget {
                target: Target::Card(old_land),
            }) {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(old_land),
                    },
                );
                if g.legal_actions().contains(&Action::FinishTargets) {
                    act(&mut g, Action::FinishTargets);
                }
                selected = true;
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert!(selected);
        let saved = g.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.object_view(old_land).zone, Zone::Battlefield);
            assert!(g.object_view(old_land).tapped);
        }
    }
}
