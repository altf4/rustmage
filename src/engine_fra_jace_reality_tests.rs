use super::*;

fn jace_fixture(g: &mut Game, player: Player) -> CardId {
    // The catalogue remains gated; install the authored card in an inert fixture.
    let id = put(g, player, "Bear Cub", Zone::Battlefield);
    g.state.objects[id as usize].card = Card::from_name("Jace, Reality Sculptor").unwrap();
    g.state.objects[id as usize].counters.loyalty = 5;
    g.register_battlefield_sources(id);
    id
}
fn activate_jace(g: &mut Game, source: CardId, ability: u16) {
    act(g, Action::AnnounceActivate { card: source, ability, counter_payment: Counters::default() });
    pay(g);
}

#[test]
fn jace_empowers_token_using_islands_at_resolution_and_survives_source_departure() {
    for player in [Player::P0, Player::P1] {
        for islands in [0, 2] {
            for departed in [false, true] {
                let mut g = scenario(player);
                let jace = jace_fixture(&mut g, player);
                put(&mut g, player.other(), "Island", Zone::Battlefield);
                if islands > 0 { put(&mut g, player, "Island", Zone::Battlefield); }
                activate_jace(&mut g, jace, 0);
                if islands > 0 { put(&mut g, player, "Island", Zone::Battlefield); }
                if departed { g.move_object(jace, Zone::Graveyard); }
                let saved = g.snapshot();
                for _ in 0..2 {
                    settle(&mut g);
                    let tokens: Vec<_> = g.state.objects.iter().enumerate().filter_map(|(id, object)|
                        (object.zone == Zone::Battlefield && object.controller == player
                            && object.card == Card::JacePlaneswalkerToken).then_some(id as CardId)).collect();
                    if islands == 0 {
                        // The zero-loyalty token is created then ceases during SBAs.
                        assert!(tokens.is_empty());
                    } else {
                        assert_eq!(tokens.len(), 1);
                        assert_eq!(g.counter_count(tokens[0], CounterKind::Loyalty), 2);
                    }
                    if !departed { assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 6); }
                    g.restore(&saved).unwrap();
                }
            }
        }
    }
}

#[test]
fn jace_empower_token_and_counters_obey_doubling_replacement() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let jace = jace_fixture(&mut g, player);
        put(&mut g, player, "Doubling Season", Zone::Battlefield);
        put(&mut g, player, "Island", Zone::Battlefield);
        activate_jace(&mut g, jace, 0);
        for _ in 0..48 {
            if g.legal_actions().iter().any(|a| matches!(a, Action::ChooseJace { .. })) { break; }
            choose(&mut g, |a| matches!(a, Action::Pass | Action::ChooseReplacement { .. }));
        }
        let choices: Vec<_> = g.legal_actions().into_iter().filter_map(|action| match action {
            Action::ChooseJace { object } => Some(object), _ => None,
        }).collect();
        assert_eq!(choices.len(), 2);
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::ChooseJace { object: choices[0] });
            settle(&mut g);
            assert_eq!(g.counter_count(choices[0].card, CounterKind::Loyalty), 2);
            assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 6);
            assert_ne!(g.object_view(choices[1].card).zone, Zone::Battlefield);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn jace_exiles_everything_above_bottom_even_if_threshold_changes_after_activation() {
    for player in [Player::P0, Player::P1] {
        for library_size in [0, 1, 3] {
            let mut g = scenario(player);
            let jace = jace_fixture(&mut g, player);
            let opponent = player.other();
            let existing = g.state.players[opponent.index()].library.clone();
            for id in existing { g.move_object(id, Zone::Exile); }
            let mut library = Vec::new();
            for _ in 0..library_size { library.push(put(&mut g, opponent, "Forest", Zone::Library)); }
            g.state.objects[jace as usize].counters.loyalty = 24;
            let action = Action::AnnounceActivate { card: jace, ability: 2, counter_payment: Counters::default() };
            reject(&mut g, action.clone());
            g.state.objects[jace as usize].counters.loyalty = 25;
            act(&mut g, action);
            pay(&mut g);
            g.state.objects[jace as usize].counters.loyalty = 1;
            let saved = g.snapshot();
            for _ in 0..2 {
                settle(&mut g);
                assert_eq!(g.state.players[opponent.index()].library, library.iter().take(1).copied().collect::<Vec<_>>().into());
                for &id in library.iter().skip(1) { assert_eq!(g.object_view(id).zone, Zone::Exile); }
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn jace_delayed_trigger_is_per_attacker_and_tracks_exact_incarnation() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let jace = jace_fixture(&mut g, player);
        let walker = put(&mut g, player, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 3;
        let first = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let second = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
        activate_jace(&mut g, jace, 1);
        settle(&mut g);
        g.move_object(jace, Zone::Graveyard);
        g.state.active = player.other();
        g.state.priority = player.other();
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: first });
        let defender = g.object_ref(walker);
        act(&mut g, Action::AttackPlaneswalker { card: second, defender });
        act(&mut g, Action::FinishAttackers);
        g.prepare_triggers();
        let count = g.state.stack.iter().chain(g.state.pending_triggers.iter()).chain(g.state.ordering_triggers.iter())
            .filter(|item| item.card == jace && matches!(item.kind, StackKind::TriggeredAbility { .. })).count();
        assert_eq!(count, 2);
        while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            choose(&mut g, |action| matches!(action, Action::PutTrigger { .. }));
        }
        let saved = g.snapshot();
        for _ in 0..2 {
            g.move_object(second, Zone::Exile);
            g.move_object(second, Zone::Battlefield);
            settle(&mut g);
            assert_eq!(stats(&g, first), (-3, 2));
            assert_eq!(stats(&g, second), (3, 3));
            g.finish_cleanup();
            assert_eq!(stats(&g, first), (2, 2));
            g.restore(&saved).unwrap();
        }
        g.state.active = player;
        g.begin_turn();
        assert!(g.state.delayed_opponent_attack_until_next_turn.is_none());
    }
}

#[test]
fn jace_copied_activation_produces_independent_delayed_effects() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let jace = jace_fixture(&mut g, player);
        let attacker = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        activate_jace(&mut g, jace, 1);
        let original = g.state.stack.last().unwrap().clone();
        let mut copier = original.clone();
        copier.controller = player;
        copier.targets = vec![BoundTarget { group: 0, target: Target::Stack(original.id), object: None }];
        g.copy_target_ability(&copier, 0);
        settle(&mut g);
        assert_eq!(g.state.delayed_opponent_attack_until_next_turn.as_ref().unwrap().len(), 2);
        g.move_object(jace, Zone::Graveyard);
        g.state.active = player.other();
        g.state.priority = player.other();
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        g.prepare_triggers();
        while matches!(g.state.pending, Pending::OrderTriggers { .. }) {
            choose(&mut g, |action| matches!(action, Action::PutTrigger { .. }));
        }
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert_eq!(stats(&g, attacker), (-8, 2));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn jace_granted_activation_retains_its_trigger_definition() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let walker = put(&mut g, player, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 5;
        let attacker = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
        let source = g.object_ref(walker);
        let timestamp = g.next_role_timestamp();
        g.grant_recipient_ability(source, Card::from_name("Jace, Reality Sculptor").unwrap(),
            SelfGrantedAbilityKind::Activated { ability: 1 }, timestamp, None);
        act(&mut g, Action::AnnounceSelfGrantedAbility { source, grant_timestamp: timestamp });
        pay(&mut g);
        settle(&mut g);
        g.move_object(walker, Zone::Graveyard);
        g.state.active = player.other();
        g.state.priority = player.other();
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        g.prepare_triggers();
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert_eq!(stats(&g, attacker), (-3, 2));
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn jace_empower_uses_live_island_control_and_chooses_between_existing_tokens() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let jace = jace_fixture(&mut g, player);
        let land = put(&mut g, player, "Island", Zone::Battlefield);
        put(&mut g, player, "Island", Zone::Battlefield);
        g.create_tokens(player, Card::JacePlaneswalkerToken, 2, false);
        let tokens: Vec<CardId> = g.state.objects.iter().enumerate().filter_map(|(id, object)|
            (object.zone == Zone::Battlefield && object.controller == player
                && object.card == Card::JacePlaneswalkerToken).then_some(id as CardId)).collect();
        assert_eq!(tokens.len(), 2);
        for &token in &tokens { g.state.objects[token as usize].counters.loyalty = 1; }
        activate_jace(&mut g, jace, 0);
        g.change_controller(g.object_ref(land), player.other());
        g.resolve_top();
        let first = g.object_ref(tokens[0]);
        let second = g.object_ref(tokens[1]);
        assert!(g.legal_actions().contains(&Action::ChooseJace { object: first }));
        assert!(g.legal_actions().contains(&Action::ChooseJace { object: second }));
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::ChooseJace { object: second });
            settle(&mut g);
            assert_eq!(g.counter_count(tokens[0], CounterKind::Loyalty), 1);
            assert_eq!(g.counter_count(tokens[1], CounterKind::Loyalty), 2);
            assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 6);
            g.restore(&saved).unwrap();
        }
    }
}
