//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

#[test]
fn null_does_not_trigger_when_not_cast_and_reveals_land_only_hand() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[source as usize].card = Card::from_name("Null Summoner").unwrap();
        let land = put(&mut g, player.other(), "Forest", Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        settle(&mut g);
        assert!(!g.observe(player).known_hand.contains(&g.object_ref(land)));
        g.move_object(source, Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: source });
        pay(&mut g);
        for _ in 0..80 {
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty()
                && g.state.resolving.is_none()
            {
                break;
            }
            if g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Player(player.other()),
            }) {
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(player.other()),
                    },
                );
            } else {
                choose(&mut g, |a| {
                    matches!(
                        a,
                        Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
                    )
                });
            }
        }
        assert!(g.observe(player).known_hand.contains(&g.object_ref(land)));
        assert_eq!(g.object_view(land).zone, Zone::Hand);
        assert!(
            g.state
                .linked_choices
                .as_ref()
                .is_none_or(|data| data.null_cast_links.is_empty())
        );
    }
}
