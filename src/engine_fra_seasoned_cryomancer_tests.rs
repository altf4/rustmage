//! Gated Seasoned Cryomancer, actual discard count and graveyard activation.
use super::*;

fn cryomancer(g: &mut Game, p: Player, zone: Zone) -> CardId {
    let card = put(g, p, "Bear Cub", zone);
    g.state.objects[card as usize].card = Card::from_name("Seasoned Cryomancer").unwrap();
    if zone == Zone::Graveyard { g.register_graveyard_sources(card); }
    card
}

#[test]
fn seasoned_discards_create_one_separate_trigger_with_actual_target_limit() {
    for player in [Player::P0, Player::P1] {
        for nonlands in 0..=2 {
            let mut g = scenario(player);
            let source = cryomancer(&mut g, player, Zone::Hand);
            let one = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
            let two = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
            let hand: Vec<_> = (0..2).map(|i| put(&mut g, player,
                if i < nonlands { "Shock" } else { "Forest" }, Zone::Hand)).collect();
            act(&mut g, Action::AnnounceCast { card: source });
            pay(&mut g);
            for _ in 0..40 {
                g.prepare_triggers();
                if matches!(g.state.pending, Pending::EffectDiscard { .. }) { break; }
                choose(&mut g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
            }
            assert!(matches!(g.state.pending, Pending::EffectDiscard { .. }));
            assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 2);
            let saved = g.snapshot();
            for _ in 0..2 {
                for &card in &hand { act(&mut g, Action::Discard { card }); }
                for _ in 0..40 {
                    g.prepare_triggers();
                    if g.legal_actions().contains(&Action::ChooseTarget { target: Target::Card(one) }) { break; }
                    if nonlands == 0 && g.state.stack.is_empty() && g.state.pending_triggers.is_empty() { break; }
                    choose(&mut g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
                }
                if nonlands > 0 {
                    act(&mut g, Action::ChooseTarget { target: Target::Card(one) });
                    if nonlands == 2 { act(&mut g, Action::ChooseTarget { target: Target::Card(two) }); }
                    if g.legal_actions().contains(&Action::FinishTargets) { act(&mut g, Action::FinishTargets); }
                    settle(&mut g);
                    assert!(g.object_view(one).tapped);
                    assert_eq!(g.counter_count(one, CounterKind::Stun), 1);
                    assert_eq!(g.counter_count(two, CounterKind::Stun), u64::from(nonlands == 2));
                } else {
                    assert_eq!(g.counter_count(one, CounterKind::Stun), 0);
                }
                g.restore(&saved).unwrap();
                assert_eq!(g.snapshot(), saved);
            }
        }
    }
}

#[test]
fn seasoned_exiles_itself_as_cost_and_draws_two_at_instant_speed() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let card = cryomancer(&mut g, player, Zone::Graveyard);
        g.state.active = player.other();
        g.state.priority = player;
        g.state.players[player.index()].mana = [0, 2, 0, 0, 0, 3];
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::AnnounceActivate { card, ability: 0, counter_payment: Counters::default() });
            pay(&mut g);
            assert_eq!(g.object_view(card).zone, Zone::Exile);
            settle(&mut g);
            assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, 2);
            g.restore(&saved).unwrap();
            assert_eq!(g.snapshot(), saved);
        }
    }
}
