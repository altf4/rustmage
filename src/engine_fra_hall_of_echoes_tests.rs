use super::*;

fn hall_activate(g: &mut Game, hall: CardId, target: CardId) {
    act(
        g,
        Action::AnnounceActivate {
            card: hall,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}

#[test]
fn hall_copies_controlled_creature_and_temporarily_suppresses_legend_rule() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let hall = put(&mut g, player, "Hall of Echoes", Zone::Battlefield);
        let legend = put(&mut g, player, "Vraska, Soul of Stone", Zone::Battlefield);
        hall_activate(&mut g, hall, legend);
        let before = g.snapshot();
        settle(&mut g);
        assert_eq!(g.object_name(hall), g.object_name(legend));
        assert!(g.is_creature(hall));
        assert!(g.observe(player).players[player.index()].legend_rule_ignored_this_turn);
        assert!(g.observe(player.other()).players[player.index()].legend_rule_ignored_this_turn);
        g.state_based_actions();
        assert_eq!(g.object_view(hall).zone, Zone::Battlefield);
        assert_eq!(g.object_view(legend).zone, Zone::Battlefield);
        g.restore(&before).unwrap();
        settle(&mut g);
        assert!(g.state.players[player.index()].legend_rule_ignored_this_turn);
        g.move_object(hall, Zone::Graveyard);
        assert!(g.state.players[player.index()].legend_rule_ignored_this_turn);
    }
}

#[test]
fn hall_invalid_target_fizzles_both_copy_and_legend_exception() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let hall = put(&mut g, player, "Hall of Echoes", Zone::Battlefield);
        let target = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        hall_activate(&mut g, hall, target);
        g.move_object(target, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_name(hall), "Hall of Echoes");
        assert!(!g.state.players[player.index()].legend_rule_ignored_this_turn);
    }
}

#[test]
fn hall_effect_survives_source_departure_and_cleanup_rechecks_legends() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let hall = put(&mut g, player, "Hall of Echoes", Zone::Battlefield);
        let target = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        hall_activate(&mut g, hall, target);
        g.move_object(hall, Zone::Graveyard);
        settle(&mut g);
        assert!(g.state.players[player.index()].legend_rule_ignored_this_turn);
        let first = put(&mut g, player, "Vraska, Soul of Stone", Zone::Battlefield);
        let second = put(&mut g, player, "Vraska, Soul of Stone", Zone::Battlefield);
        g.state_based_actions();
        assert_eq!(g.object_view(first).zone, Zone::Battlefield);
        assert_eq!(g.object_view(second).zone, Zone::Battlefield);
        g.finish_cleanup();
        assert!(!g.state.players[player.index()].legend_rule_ignored_this_turn);
        assert!(matches!(g.state.pending, Pending::Legend { player: p } if p == player));
    }
}
