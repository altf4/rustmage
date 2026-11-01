use super::*;

fn curse_fixture(g: &mut Game, player: Player) -> CardId {
    let id = put(g, player, "Bear Cub", Zone::Battlefield);
    g.state.objects[id as usize].card = Card::from_name("Garruk, Curse Breaker").unwrap();
    g.state.objects[id as usize].counters.loyalty = 5;
    g.register_battlefield_sources(id);
    id
}
fn curse_activate(g: &mut Game, source: CardId, ability: u16) {
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
fn blank(g: &mut Game, player: Player, source: CardId, target: CardId) {
    let mut item = g.activation_item(player, source, 0);
    item.targets = vec![BoundTarget {
        group: 0,
        target: Target::Card(target),
        object: Some(g.object_ref(target)),
    }];
    g.remove_target_abilities_until_end_of_turn(&item);
}

#[test]
fn garruk_blanking_before_activation_prevents_it_but_after_does_not_cancel_it() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = curse_fixture(&mut g, player);
        let before = g.snapshot();
        blank(&mut g, player, source, source);
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::AnnounceActivate { card, .. } if *card == source))
        );
        g.restore(&before).unwrap();
        curse_activate(&mut g, source, 2);
        blank(&mut g, player, source, source);
        settle(&mut g);
        assert_eq!(
            g.state
                .delayed_opponent_attack_until_next_turn
                .as_ref()
                .unwrap()
                .len(),
            1
        );
    }
}

#[test]
fn garruk_delayed_trample_obeys_ability_removal_timestamps() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = curse_fixture(&mut g, player);
        let attacker = put(&mut g, player, "Bear Cub", Zone::Battlefield);
        curse_activate(&mut g, source, 2);
        settle(&mut g);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        g.prepare_triggers();
        blank(&mut g, player, source, attacker);
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert_eq!(stats(&g, attacker), (4, 4));
            assert!(g.keywords(attacker).contains(Keywords::TRAMPLE));
            blank(&mut g, player, source, attacker);
            assert!(!g.keywords(attacker).contains(Keywords::TRAMPLE));
            assert_eq!(stats(&g, attacker), (4, 4));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn garruk_beast_identity_is_visible_to_both_players() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = curse_fixture(&mut g, player);
        curse_activate(&mut g, source, 1);
        settle(&mut g);
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find_map(|(id, object)| {
                (object.zone == Zone::Battlefield && object.card == Card::test_index(5239))
                    .then_some(id as CardId)
            })
            .unwrap();
        for viewer in [player, player.other()] {
            let view = g
                .observe(viewer)
                .objects
                .into_iter()
                .find(|view| view.id == token)
                .unwrap();
            assert_eq!(view.name(), "Beast Token");
            assert_eq!(view.colors, 16);
            assert!(view.has_subtype("Beast"));
            assert!(view.keywords.contains(Keywords::TRAMPLE));
        }
    }
}

#[test]
fn garruk_duration_ignores_skipped_turn_and_ends_on_actual_extra_turn() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = curse_fixture(&mut g, player);
        curse_activate(&mut g, source, 2);
        settle(&mut g);
        g.add_extra_turn(player.other(), false);
        g.advance_to_actual_turn();
        assert_eq!(g.state.active, player.other());
        g.begin_turn();
        assert!(g.state.delayed_opponent_attack_until_next_turn.is_some());
        // Consume the ordinary successor after the extra turn.
        g.advance_to_actual_turn();
        g.begin_turn();
        assert!(g.state.delayed_opponent_attack_until_next_turn.is_some());
        let mut skips = [0; 2];
        skips[player.index()] = 1;
        g.state.turn_skips = Some(Box::new(skips));
        g.advance_to_actual_turn();
        assert_eq!(g.state.active, player.other());
        g.begin_turn();
        assert!(g.state.delayed_opponent_attack_until_next_turn.is_some());
        g.add_extra_turn(player, false);
        g.advance_to_actual_turn();
        assert_eq!(g.state.active, player);
        g.begin_turn();
        assert!(g.state.delayed_opponent_attack_until_next_turn.is_none());
    }
}
