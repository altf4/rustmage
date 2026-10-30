use super::*;

fn theorist_activate(g: &mut Game, source: CardId, ability: u16) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
    pay(g);
}

#[test]
fn theorist_minus_six_finishes_after_miracle_and_source_dies_from_loyalty_cost() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(
            &mut g,
            player,
            "The Theorist, Jace Beleren",
            Zone::Battlefield,
        );
        g.state.objects[source as usize].counters.loyalty = 6;
        let creature = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        put(&mut g, player, "Lorehold, the Historian", Zone::Battlefield);
        put(&mut g, player, "Island", Zone::Library);
        put(&mut g, player, "Island", Zone::Library);
        let miracle = put(&mut g, player, "Group Project", Zone::Library);
        theorist_activate(&mut g, source, 2);
        for _ in 0..48 {
            if matches!(g.state.pending, Pending::MiracleReveal { .. }) {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert!(matches!(g.state.pending, Pending::MiracleReveal { player: p } if p == player));
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 0);
        assert_eq!(g.observe(player).miracle.unwrap().object.card, miracle);
        assert!(g.observe(player.other()).miracle.is_none());
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::DeclineMiracle);
            settle(&mut g);
            assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 3);
            assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 3);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn theorist_minus_six_counts_replacement_increased_draws_at_resolution() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(
            &mut g,
            player,
            "The Theorist, Jace Beleren",
            Zone::Battlefield,
        );
        g.state.objects[source as usize].counters.loyalty = 7;
        let creature = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        put(&mut g, player, "Bard, King of Dale", Zone::Battlefield);
        for _ in 0..9 {
            put(&mut g, player, "Island", Zone::Library);
        }
        theorist_activate(&mut g, source, 2);
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 6);
            assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 6);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn theorist_minus_two_rechecks_target_controller_at_resolution() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(
            &mut g,
            player,
            "The Theorist, Jace Beleren",
            Zone::Battlefield,
        );
        g.state.objects[source as usize].counters.loyalty = 3;
        let target = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        act(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        g.change_controller(g.object_ref(target), player);
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Battlefield);
            assert_eq!(g.object_view(target).controller, player);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn theorist_illusion_name_and_color_are_visible_to_both_players() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(
            &mut g,
            player,
            "The Theorist, Jace Beleren",
            Zone::Battlefield,
        );
        g.state.objects[source as usize].counters.loyalty = 3;
        theorist_activate(&mut g, source, 0);
        settle(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, object)| {
                (object.zone == Zone::Battlefield && object.card == Card::test_index(5464))
                    .then_some(id as CardId)
            })
            .unwrap();
        for viewer in [Player::P0, Player::P1] {
            let observation = g.observe(viewer);
            let view = observation
                .objects
                .iter()
                .find(|object| object.id == token)
                .unwrap();
            assert_eq!(view.name(), "Illusion Token");
            assert_eq!(view.colors, 2);
            assert_eq!(view.controller, player);
            assert!(view.has_subtype("Illusion"));
        }
    }
}

#[test]
fn copied_theorist_triggers_after_opponents_real_turn_based_draw() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player.other());
        let source = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.apply_temporary_copy(
            g.object_ref(source),
            CopyRecipe::new(Card::from_name("The Theorist, Jace Beleren").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        g.state.objects[source as usize].counters.loyalty = 3;
        g.begin_step(Step::Upkeep);
        g.state.pending = Pending::Priority;
        g.state.priority = player.other();
        let before = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::Pass);
            act(&mut g, Action::Pass);
            assert_eq!(g.state.step, Step::Draw);
            assert_eq!(
                g.state.players[player.other().index()].cards_drawn_this_turn,
                1
            );
            assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 0);
            g.change_controller(g.object_ref(source), player.other());
            settle(&mut g);
            assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 1);
            assert_eq!(
                g.state.players[player.other().index()].cards_drawn_this_turn,
                1
            );
            g.restore(&before).unwrap();
        }
    }
}

#[test]
fn theorist_short_library_uses_actual_hand_before_empty_draw_loss() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(
            &mut g,
            player,
            "The Theorist, Jace Beleren",
            Zone::Battlefield,
        );
        g.state.objects[source as usize].counters.loyalty = 7;
        let creature = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.players[player.index()].library.clear();
        put(&mut g, player, "Island", Zone::Library);
        theorist_activate(&mut g, source, 2);
        for _ in 0..48 {
            if g.state.resolving.is_none() && g.state.stack.is_empty() {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 1);
        assert!(g.state.players[player.index()].attempted_empty_draw);
        assert_eq!(g.counter_count(creature, CounterKind::PlusOne), 1);
    }
}
