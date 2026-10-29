use super::*;

#[test]
fn emrakul_cast_trigger_untaps_controlled_lands_even_after_spell_is_countered() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let land = put(&mut g, player, "Forest", Zone::Battlefield);
        let enemy = put(&mut g, player.other(), "Island", Zone::Battlefield);
        let creature = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        g.state.objects[land as usize].tapped = true;
        g.state.objects[enemy as usize].tapped = true;
        g.state.objects[creature as usize].tapped = true;
        let spell = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[spell as usize].card = Card::from_name("Emrakul, the Exigent Doom").unwrap();
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 10];
        act(&mut g, Action::AnnounceCast { card: spell });
        pay(&mut g);
        g.prepare_triggers();
        let stack_id = g.state.stack.iter().find(|item| item.kind == StackKind::Spell && item.card == spell).unwrap().id;
        assert!(g.counter_stack_item(stack_id, false));
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert!(!g.object_view(land).tapped);
            assert!(g.object_view(enemy).tapped);
            assert!(g.object_view(creature).tapped);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn emrakul_cast_untap_uses_live_control_types_and_stun() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let stolen = put(&mut g, player.other(), "Forest", Zone::Battlefield);
        let blocked = put(&mut g, player, "Island", Zone::Battlefield);
        let stolen_ref = g.object_ref(stolen);
        g.change_controller(stolen_ref, player);
        g.state.objects[stolen as usize].tapped = true;
        g.state.objects[blocked as usize].tapped = true;
        g.state.objects[blocked as usize].counters.stun = 1;
        let spell = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[spell as usize].card = Card::from_name("Emrakul, the Exigent Doom").unwrap();
        g.state.players[player.index()].mana = [0, 0, 0, 0, 0, 10];
        act(&mut g, Action::AnnounceCast { card: spell });
        pay(&mut g);
        g.prepare_triggers();
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert!(!g.object_view(stolen).tapped);
            assert!(g.object_view(blocked).tapped);
            assert_eq!(g.counter_count(blocked, CounterKind::Stun), 0);
            assert_eq!(stats(&g, spell), (12, 12));
            assert!(g.keywords(spell).contains(Keywords::FLYING.union(Keywords::TRAMPLE)));
            g.restore(&saved).unwrap();
        }
    }
}
