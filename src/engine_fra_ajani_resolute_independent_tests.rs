use super::*;

fn ajani_independent_finish(g: &mut Game) {
    for _ in 0..128 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty() && g.state.resolving.is_none() { return; }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }
            | Action::ChooseReplacement { .. } | Action::ChooseZoneReplacement { .. }));
    }
    panic!("Ajani continuation did not finish: {:?}", g.state.pending);
}

#[test]
fn ajani_resolute_real_cast_requires_white_and_enters_with_two_loyalty() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let ajani = put(&mut g, player, "Ajani Resolute", Zone::Hand);
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 2];
        let saved = g.snapshot();
        reject(&mut g, Action::AnnounceCast { card: ajani });
        assert_eq!(g.snapshot(), saved);
        g.state.players[player.index()].mana = [1, 0, 0, 0, 0, 1];
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::AnnounceCast { card: ajani });
            pay(&mut g);
            assert_eq!(g.state.players[player.index()].mana, [0; 6]);
            ajani_independent_finish(&mut g);
            assert_eq!(g.object_view(ajani).zone, Zone::Battlefield);
            assert_eq!(g.counter_count(ajani, CounterKind::Loyalty), 2);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn ajani_doubled_named_cats_and_life_triggers_follow_live_controller() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        put(&mut g, player, "Doubling Season", Zone::Battlefield);
        let ajani = put(&mut g, player, "Ajani Resolute", Zone::Battlefield);
        g.state.objects[ajani as usize].counters.loyalty = 5;
        act(&mut g, Action::AnnounceActivate { card: ajani, ability: 1, counter_payment: Counters::default() });
        pay(&mut g);
        ajani_independent_finish(&mut g);
        let cats: Vec<CardId> = g.state.objects.iter().enumerate().filter_map(|(id, o)|
            (o.is_token() && o.zone == Zone::Battlefield && o.card == Card::test_index(5462))
                .then_some(id as CardId)).collect();
        assert_eq!(cats.len(), 2);
        for &cat in &cats {
            assert_eq!(g.object_view(cat).controller, player);
            assert_eq!(g.object_view(cat).name(), "Ajani's Pridemate");
            assert_eq!(g.object_view(cat).colors, 1);
            assert!(g.has_subtype(cat, "Cat"));
            assert!(g.has_subtype(cat, "Soldier"));
            assert_eq!(g.stats(cat).power, 2);
            assert_eq!(g.stats(cat).toughness, 2);
            for viewer in [Player::P0, Player::P1] {
                let view = g.observe(viewer);
                let object = view.objects.iter().find(|o| o.reference.card == cat).unwrap();
                assert_eq!(object.name(), "Ajani's Pridemate");
                assert_eq!(object.colors, 1);
            }
        }
        let saved = g.snapshot();
        for _ in 0..2 {
            g.gain_life(player, 1);
            g.gain_life(player, 2);
            g.change_controller(g.object_ref(cats[0]), player.other());
            ajani_independent_finish(&mut g);
            assert_eq!(g.counter_count(ajani, CounterKind::Loyalty), 5);
            assert_eq!(g.counter_count(cats[0], CounterKind::PlusOne), 2);
            assert_eq!(g.counter_count(cats[1], CounterKind::PlusOne), 4);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn ajani_stacked_emblems_apply_to_new_creatures_and_follow_live_control() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let ajani = put(&mut g, player, "Ajani Resolute", Zone::Battlefield);
        let mine = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        let theirs = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[ajani as usize].counters.loyalty = 11;
        act(&mut g, Action::AnnounceActivate { card: ajani, ability: 2, counter_payment: Counters::default() });
        pay(&mut g);
        ajani_independent_finish(&mut g);
        g.create_emblem(player, Card::test_index(5463));
        g.move_object(ajani, Zone::Graveyard);
        let later = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        assert_eq!(g.stats(mine).power, 6);
        assert_eq!(g.stats(later).toughness, 6);
        assert_eq!(g.stats(theirs).power, 2);
        let saved = g.snapshot();
        for _ in 0..2 {
            g.change_controller(g.object_ref(mine), player.other());
            g.change_controller(g.object_ref(theirs), player);
            assert_eq!(g.stats(mine).power, 2);
            assert_eq!(g.stats(theirs).power, 6);
            assert_eq!(g.stats(later).power, 6);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn ajani_life_observer_follows_current_control_and_queued_trigger_keeps_source() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let ajani = put(&mut g, player, "Ajani Resolute", Zone::Battlefield);
        g.state.objects[ajani as usize].counters.loyalty = 2;
        g.gain_life(player, 1);
        g.change_controller(g.object_ref(ajani), player.other());
        g.gain_life(player, 1);
        g.gain_life(player.other(), 1);
        let saved = g.snapshot();
        for _ in 0..2 {
            ajani_independent_finish(&mut g);
            assert_eq!(g.counter_count(ajani, CounterKind::Loyalty), 4);
            g.restore(&saved).unwrap();
        }
    }
}
