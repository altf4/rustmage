//! Jace's printed abilities through normal casts, choices and owned replay.
use super::big_helpers::{cast as paid_cast, checked, next_turn, settle as finish};
use super::*;

#[test]
fn jace_reawakened_full_card() {
    for player in [Player::P0, Player::P1] {
        let mut loot = scenario(player);
        let jace = put(&mut loot, player, "Jace Reawakened", Zone::Hand);
        for turn in 1..=3 {
            loot.state.players[player.index()].turns_started = turn;
            assert!(
                !loot
                    .legal_actions()
                    .contains(&Action::AnnounceCast { card: jace })
            );
        }
        loot.state.players[player.index()].turns_started = 4;
        paid_cast(&mut loot, jace, None, &[]);
        finish(&mut loot);
        assert_eq!(loot.counter_count(jace, CounterKind::Loyalty), 3);
        let discard = put(&mut loot, player, "Bear Cub", Zone::Hand);
        checked(
            &mut loot,
            Action::AnnounceActivate {
                card: jace,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut loot);
        loot.resolve_top();
        checked(&mut loot, Action::Discard { card: discard });
        finish(&mut loot);
        assert_eq!(loot.object_view(discard).zone, Zone::Graveyard);
        assert_eq!(loot.hand_count(player), 1);
        assert_eq!(loot.counter_count(jace, CounterKind::Loyalty), 4);

        let mut plot = scenario(player);
        let jace = put(&mut plot, player, "Jace Reawakened", Zone::Battlefield);
        plot.state.objects[jace as usize].counters.loyalty = 3;
        let creature = put(&mut plot, player, "Bear Cub", Zone::Hand);
        let forest = put(&mut plot, player, "Forest", Zone::Hand);
        checked(
            &mut plot,
            Action::AnnounceActivate {
                card: jace,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut plot);
        plot.resolve_top();
        assert!(plot.observe(player.other()).effect_choice.is_none());
        assert!(
            !plot
                .legal_actions()
                .contains(&Action::SelectResolutionExile {
                    object: plot.object_ref(forest),
                })
        );
        let object = plot.object_ref(creature);
        checked(&mut plot, Action::SelectResolutionExile { object });
        finish(&mut plot);
        assert_eq!(plot.object_view(creature).zone, Zone::Exile);
        assert!(
            !plot
                .legal_actions()
                .contains(&Action::AnnouncePlotted { card: creature })
        );
        next_turn(&mut plot);
        next_turn(&mut plot);
        assert!(
            plot.legal_actions()
                .contains(&Action::AnnouncePlotted { card: creature })
        );
        checked(&mut plot, Action::AnnouncePlotted { card: creature });
        pay(&mut plot);
        finish(&mut plot);
        assert_eq!(plot.object_view(creature).zone, Zone::Battlefield);

        let mut copies = scenario(player);
        let jace = put(&mut copies, player, "Jace Reawakened", Zone::Battlefield);
        copies.state.objects[jace as usize].counters.loyalty = 6;
        checked(
            &mut copies,
            Action::AnnounceActivate {
                card: jace,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut copies);
        finish(&mut copies);
        assert_eq!(copies.object_view(jace).zone, Zone::Graveyard);
        let bear = put(&mut copies, player, "Bear Cub", Zone::Hand);
        paid_cast(&mut copies, bear, None, &[]);
        finish(&mut copies);
        assert_eq!(
            copies
                .state
                .objects
                .iter()
                .filter(|o| { o.zone == Zone::Battlefield && o.card.name() == "Bear Cub" })
                .count(),
            2
        );
        let shock = put(&mut copies, player, "Shock", Zone::Hand);
        paid_cast(&mut copies, shock, None, &[Target::Player(player.other())]);
        copies.prepare_triggers();
        copies.resolve_top();
        checked(
            &mut copies,
            Action::SetStackTarget {
                index: 0,
                target: Target::Player(player),
            },
        );
        checked(&mut copies, Action::FinishStackTargets);
        finish(&mut copies);
        assert_eq!(copies.state.players[player.index()].life, 18);
        assert_eq!(copies.state.players[player.other().index()].life, 18);
    }
}
