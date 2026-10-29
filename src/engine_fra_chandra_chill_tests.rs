use super::*;

fn chill(g: &mut Game, player: Player, loyalty: u64) -> CardId {
    let id = put(g, player, "Bear Cub", Zone::Battlefield);
    g.state.objects[id as usize].card = Card::from_name("Chandra, Chill of Compliance").unwrap();
    g.state.objects[id as usize].counters.loyalty = loyalty;
    g.register_battlefield_sources(id);
    id
}
fn activate(g: &mut Game, id: CardId, ability: u16) {
    act(g, Action::AnnounceActivate { card: id, ability, counter_payment: Counters::default() });
    pay(g);
}

#[test]
fn chill_surveil_returns_only_noncreature_nonland_actually_moved_to_graveyard() {
    for player in [Player::P0, Player::P1] {
        for (name, replacement, expected) in [
            ("Shock", false, Zone::Hand), ("Bear Cub", false, Zone::Graveyard),
            ("Forest", false, Zone::Graveyard), ("Shock", true, Zone::Exile),
        ] {
            let mut g = scenario(player);
            let source = chill(&mut g, player, 3);
            if replacement { put(&mut g, player, "Rest in Peace", Zone::Battlefield); }
            let card = put(&mut g, player, name, Zone::Library);
            activate(&mut g, source, 0);
            g.move_object(source, Zone::Graveyard);
            g.resolve_top();
            assert!(matches!(g.state.pending, Pending::EffectLibrary { .. }));
            let saved = g.snapshot();
            for _ in 0..2 {
                act(&mut g, Action::ArrangeLibrary { card, destination: LibraryDestination::Graveyard });
                assert_eq!(g.object_view(card).zone, expected);
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn chill_restricted_blue_funds_noncreature_spell_but_not_creature() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = chill(&mut g, player, 3);
        g.state.players[player.index()].mana = [0; 6];
        activate(&mut g, source, 1);
        g.resolve_top();
        assert_eq!(g.state.players[player.index()].mana[1], 1);
        assert!(g.mana_lots(player).iter().any(|lot| lot.quality.restriction == ManaRestrictionView::NoncreatureSpell));
        let creature = put(&mut g, player, "Spectral Sailor", Zone::Hand);
        reject(&mut g, Action::AnnounceCast { card: creature });
        let spell = put(&mut g, player, "Opt", Zone::Hand);
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::AnnounceCast { card: spell });
            pay(&mut g);
            assert_eq!(g.state.players[player.index()].mana[1], 0);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn chill_variable_loyalty_cost_keeps_x_after_source_leaves() {
    for player in [Player::P0, Player::P1] {
        for amount in [0, 1, 3] {
            let mut g = scenario(player);
            let source = chill(&mut g, player, 3);
            let target = put(&mut g, player.other(), "Gilded Lotus", Zone::Battlefield);
            let source_snapshot = g.snapshot();
            act(&mut g, Action::AnnounceActivate { card: source, ability: 2, counter_payment: Counters::default() });
            reject(&mut g, Action::ChooseSourceCounterAmount { amount: 4 });
            act(&mut g, Action::ChooseSourceCounterAmount { amount });
            act(&mut g, Action::ChooseTarget { target: Target::Card(target) });
            if g.legal_actions().contains(&Action::FinishTargets) { act(&mut g, Action::FinishTargets); }
            pay(&mut g);
            let item = g.state.stack.last().unwrap();
            assert_eq!(item.x, Some(amount as u32));
            assert_eq!(item.loyalty_counters_paid, amount);
            if amount != 3 { g.move_object(source, Zone::Graveyard); }
            let saved = g.snapshot();
            for _ in 0..2 {
                g.resolve_top();
                assert!(g.object_view(target).tapped);
                assert_eq!(g.counter_count(target, CounterKind::Stun), amount);
                g.restore(&saved).unwrap();
            }
            g.restore(&source_snapshot).unwrap();
        }
    }
}

#[test]
fn chill_emblem_draws_after_source_departure_on_own_cast() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = chill(&mut g, player, 7);
        activate(&mut g, source, 3);
        g.resolve_top();
        g.move_object(source, Zone::Graveyard);
        assert!(g.state.objects.iter().any(|o| o.zone == Zone::Command && o.card == Card::test_index(5466)));
        let creature = put(&mut g, player, "Bear Cub", Zone::Hand);
        let draws = g.state.players[player.index()].cards_drawn_this_turn;
        cast(&mut g, creature, None);
        g.prepare_triggers();
        g.resolve_top();
        assert_eq!(g.state.players[player.index()].cards_drawn_this_turn, draws + 1);
    }
}

#[test]
fn chill_copied_x_activation_preserves_paid_amount_and_target_incarnation() {
    for player in [Player::P0, Player::P1] {
        for reenter in [false, true] {
            let mut g = scenario(player);
            let source = chill(&mut g, player, 3);
            let target = put(&mut g, player.other(), "Gilded Lotus", Zone::Battlefield);
            act(&mut g, Action::AnnounceActivate { card: source, ability: 2, counter_payment: Counters::default() });
            act(&mut g, Action::ChooseSourceCounterAmount { amount: 2 });
            act(&mut g, Action::ChooseTarget { target: Target::Card(target) });
            pay(&mut g);
            let original = g.state.stack.last().unwrap().clone();
            let mut copier = original.clone();
            copier.targets = vec![BoundTarget { group: 0, target: Target::Stack(original.id), object: None }];
            g.copy_target_ability(&copier, 0);
            if g.legal_actions().contains(&Action::FinishStackTargets) { act(&mut g, Action::FinishStackTargets); }
            let copied = g.state.stack.last().unwrap();
            assert!(copied.is_copy);
            assert_eq!(copied.x, Some(2));
            g.move_object(source, Zone::Graveyard);
            if reenter {
                g.move_object(target, Zone::Exile);
                g.move_object(target, Zone::Battlefield);
            }
            let saved = g.snapshot();
            for _ in 0..2 {
                settle(&mut g);
                assert_eq!(g.counter_count(target, CounterKind::Stun), if reenter { 0 } else { 4 });
                assert_eq!(g.object_view(target).tapped, !reenter);
                g.restore(&saved).unwrap();
            }
        }
    }
}

#[test]
fn chill_large_loyalty_x_survives_copy_observation_and_restore() {
    let amount = u64::from(u32::MAX) + 17;
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = chill(&mut g, player, amount + 1);
        let target = put(&mut g, player.other(), "Gilded Lotus", Zone::Battlefield);
        let initial = g.snapshot();
        act(&mut g, Action::AnnounceActivate { card: source, ability: 2, counter_payment: Counters::default() });
        assert_eq!(g.source_counter_choice().unwrap().maximum, amount + 1);
        reject(&mut g, Action::ChooseSourceCounterAmount { amount: amount + 2 });
        act(&mut g, Action::ChooseSourceCounterAmount { amount });
        act(&mut g, Action::ChooseTarget { target: Target::Card(target) });
        if g.legal_actions().contains(&Action::FinishTargets) { act(&mut g, Action::FinishTargets); }
        pay(&mut g);
        let original = g.state.stack.last().unwrap().clone();
        assert_eq!(original.loyalty_x, Some(amount));
        assert_eq!(original.loyalty_counters_paid, amount);
        for viewer in [player, player.other()] {
            let json = serde_json::to_value(g.observe(viewer)).unwrap();
            assert!(json.to_string().contains(&format!("\"loyalty_x\":{amount}")));
        }
        let mut copier = original.clone();
        copier.targets = vec![BoundTarget { group: 0, target: Target::Stack(original.id), object: None }];
        g.copy_target_ability(&copier, 0);
        if g.legal_actions().contains(&Action::FinishStackTargets) { act(&mut g, Action::FinishStackTargets); }
        assert_eq!(g.state.stack.last().unwrap().loyalty_x, Some(amount));
        g.move_object(source, Zone::Graveyard);
        let saved = g.snapshot();
        for _ in 0..2 {
            settle(&mut g);
            assert_eq!(g.counter_count(target, CounterKind::Stun), amount * 2);
            g.restore(&saved).unwrap();
        }
        g.restore(&initial).unwrap();
        assert_eq!(g.counter_count(source, CounterKind::Loyalty), amount + 1);
        assert_eq!(g.counter_count(target, CounterKind::Stun), 0);
    }
}
