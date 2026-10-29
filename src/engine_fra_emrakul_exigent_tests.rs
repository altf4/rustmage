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

#[test]
fn emrakul_ward_sacrifice_three_is_selected_before_atomic_payment() {
    for player in [Player::P0, Player::P1] {
        for pay_ward in [false, true] {
            let mut g = scenario(player);
            let emrakul = put(&mut g, player, "Bear Cub", Zone::Battlefield);
            g.state.objects[emrakul as usize].card = Card::from_name("Emrakul, the Exigent Doom").unwrap();
            g.register_battlefield_sources(emrakul);
            let payer = player.other();
            let permanents: Vec<_> = (0..3).map(|_| put(&mut g, payer, "Forest", Zone::Battlefield)).collect();
            let shock = put(&mut g, payer, "Shock", Zone::Hand);
            g.state.active = payer;
            g.state.priority = payer;
            cast(&mut g, shock, Some(Target::Card(emrakul)));
            for _ in 0..48 {
                if matches!(g.state.pending, Pending::EffectChoice { player: p } if p == payer) { break; }
                choose(&mut g, |action| matches!(action, Action::Pass | Action::PutTrigger { .. }));
            }
            assert!(matches!(g.state.pending, Pending::EffectChoice { player: p } if p == payer));
            let saved = g.snapshot();
            for _ in 0..2 {
                if pay_ward {
                    for (i, id) in permanents.iter().copied().enumerate() {
                        let reference = g.object_ref(id);
                        act(&mut g, Action::Sacrifice { object: reference });
                        if i < 2 { assert!(permanents.iter().all(|&id| g.object_view(id).zone == Zone::Battlefield)); }
                    }
                } else {
                    let reference = g.object_ref(permanents[0]);
                    act(&mut g, Action::Sacrifice { object: reference });
                    act(&mut g, Action::DeclineEffectPayment);
                }
                settle(&mut g);
                for &id in &permanents {
                    assert_eq!(g.object_view(id).zone, if pay_ward { Zone::Graveyard } else { Zone::Battlefield });
                }
                assert_eq!(g.object_view(emrakul).damage, if pay_ward { 2 } else { 0 });
                g.restore(&saved).unwrap();
            }
        }
    }
}
