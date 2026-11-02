//! Printed max-speed artifact effects, ordinary costs and exact draw/discard order.
mod support;
use rustmage as _;
use rustmage_core::engine::*;
use support::*;

fn drain(g: &mut Game) {
    for _ in 0..48 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("bounded speed artifact resolution");
}
fn reach_discard(g: &mut Game) {
    for _ in 0..48 {
        if g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::Discard { .. }))
        {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("draw must reach the mandatory discard");
}
fn hand(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.owner == p && o.zone == Zone::Hand)
        .map(|(i, _)| i as CardId)
        .collect()
}

#[test]
fn all_three_artifacts_start_speed_only_on_their_actual_paid_entry() {
    for p in [Player::P0, Player::P1] {
        for (name, cost) in [
            ("Racers' Scoreboard", 4),
            ("Starting Column", 3),
            ("Walking Sarcophagus", 2),
        ] {
            let mut g = scenario(p);
            let source = put(&mut g, p, name, Zone::Hand);
            assert_eq!(g.state.players[p.index()].speed, 0);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, cost];
            cast(&mut g, source, None);
            if name == "Racers' Scoreboard" {
                reach_discard(&mut g);
                assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
                let card = hand(&g, p)[0];
                act(&mut g, Action::Discard { card });
            }
            drain(&mut g);
            assert_eq!(g.object_view(source).zone, Zone::Battlefield);
            assert_eq!(g.state.players[p.index()].speed, 1, "{name}");
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            if name == "Walking Sarcophagus" {
                assert_eq!(stats(&g, source), (2, 1));
            }
        }
    }
}

#[test]
fn scoreboard_draws_two_before_choosing_exactly_one_discard_even_after_it_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Racers' Scoreboard", Zone::Hand);
        let old = put(&mut g, p, "Hill Giant", Zone::Hand);
        let foreign = put(&mut g, p.other(), "Forest", Zone::Hand);
        cast(&mut g, source, None);
        for _ in 0..24 {
            if g.state.objects[source as usize].zone == Zone::Battlefield {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        g.move_object(source, Zone::Graveyard);
        reach_discard(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), 6);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert_eq!(hand(&g, p).len(), 3);
        reject(&mut g, Action::Discard { card: foreign });
        act(&mut g, Action::Discard { card: old });
        drain(&mut g);
        assert_eq!(g.object_view(old).zone, Zone::Graveyard);
        assert_eq!(hand(&g, p).len(), 2);
        assert_eq!(g.object_view(foreign).zone, Zone::Hand);
    }
}

#[test]
fn scoreboard_discount_is_live_own_controller_generic_only_and_requires_max_speed() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Racers' Scoreboard", Zone::Battlefield);
        let creature = put(&mut g, p, "Hill Giant", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
        g.state.players[p.index()].speed = 3;
        reject(&mut g, Action::AnnounceCast { card: creature });
        g.state.players[p.index()].speed = 4;
        g.state.players[p.other().index()].speed = 4;
        g.state.objects[source as usize].controller = p.other();
        reject(&mut g, Action::AnnounceCast { card: creature });
        g.state.objects[source as usize].controller = p;
        cast(&mut g, creature, None);
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
        let strike = put(&mut g, p, "Lightning Strike", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        reject(&mut g, Action::AnnounceCast { card: strike });
        g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
        cast(&mut g, strike, Some(Target::Player(p.other())));
        drain(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.state.players[p.other().index()].life, 17);
    }
}

#[test]
fn starting_column_produces_each_color_and_max_speed_sacrifices_before_draw_two_discard_one() {
    for p in [Player::P0, Player::P1] {
        for color in 0..5 {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Starting Column", Zone::Battlefield);
            g.state.players[p.index()].mana = [0; 6];
            let mut production = [0; 6];
            production[color] = 1;
            choose(
                &mut g,
                |a| matches!(a,Action::ActivateMana {card,ability:0,production:m,..}if *card==source&&*m==production),
            );
            assert_eq!(g.state.players[p.index()].mana, production);
            assert!(g.object_view(source).tapped);
        }
        let mut g = scenario(p);
        let source = put(&mut g, p, "Starting Column", Zone::Battlefield);
        let action = Action::AnnounceSourceGrant {
            source: g.object_ref(source),
            ability: 1,
        };
        g.state.players[p.index()].speed = 3;
        reject(&mut g, action.clone());
        g.state.players[p.index()].speed = 4;
        g.state.step = Step::Upkeep;
        act(&mut g, action);
        pay(&mut g);
        assert_eq!(g.object_view(source).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
        reach_discard(&mut g);
        assert_eq!(hand(&g, p).len(), 2);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        let card = hand(&g, p)[1];
        act(&mut g, Action::Discard { card });
        drain(&mut g);
        assert_eq!(hand(&g, p).len(), 1);
        assert_eq!(g.object_view(card).zone, Zone::Graveyard);
    }
}

#[test]
fn walking_sarcophagus_uses_only_its_live_controllers_max_speed_and_bonus_is_not_cumulative() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = put(&mut g, p, "Walking Sarcophagus", Zone::Battlefield);
        let other = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        for speed in [1, 3, 4, 4] {
            g.state.players[p.index()].speed = speed;
            assert_eq!(stats(&g, source), if speed == 4 { (3, 3) } else { (2, 1) });
            assert_eq!(stats(&g, other), (2, 2));
        }
        g.state.objects[source as usize].controller = p.other();
        assert_eq!(stats(&g, source), (2, 1));
        g.state.players[p.other().index()].speed = 4;
        assert_eq!(stats(&g, source), (3, 3));
        assert!(g.has_subtype(source, "Cat") && g.has_subtype(source, "Zombie"));
        assert_eq!(g.object_view(source).colors, 0);
    }
}
