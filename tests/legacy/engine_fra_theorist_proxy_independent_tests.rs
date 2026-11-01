use super::*;

fn grant(g: &mut Game, player: Player) {
    let proxy = put(g, player, "Theorist's Proxy", Zone::Battlefield);
    act(
        g,
        Action::AnnounceActivate {
            card: proxy,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
    settle(g);
}

#[test]
fn multiple_proxy_grants_apply_to_one_next_spell_even_after_control_changes() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        grant(&mut g, player);
        grant(&mut g, player);
        let first = put(&mut g, player, "Shock", Zone::Hand);
        cast(&mut g, first, Some(Target::Player(player.other())));
        let first_id = g.state.stack.last().unwrap().id;
        g.state.stack.last_mut().unwrap().controller = player.other();
        let saved = g.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            assert!(!g.counter_stack_item(first_id, false));
            settle(&mut g);
            g.state.priority = player;
            let second = put(&mut g, player, "Shock", Zone::Hand);
            cast(&mut g, second, Some(Target::Player(player.other())));
            let second_id = g.state.stack.last().unwrap().id;
            assert!(g.counter_stack_item(second_id, false));
        }
    }
}

#[test]
fn proxy_does_not_prevent_spell_failing_for_illegal_target() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        grant(&mut g, player);
        let target = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, player, "Shock", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(target)));
        let id = g.state.stack.last().unwrap().id;
        assert!(!g.counter_stack_item(id, false));
        g.move_object(target, Zone::Exile);
        let saved = g.snapshot();
        for replay in 0..2 {
            if replay == 1 {
                g.restore(&saved).unwrap();
            }
            settle(&mut g);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert_eq!(g.object_view(target).zone, Zone::Exile);
            assert!(!g.observe(player).uncounterable_spell_ids.contains(&id));
        }
    }
}
