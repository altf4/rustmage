use super::*;

fn cast_null(g: &mut Game, player: Player) -> CardId {
    let source = put(g, player, "Bear Cub", Zone::Hand);
    g.state.objects[source as usize].card = Card::from_name("Null Summoner").unwrap();
    act(g, Action::AnnounceCast { card: source });
    pay(g);
    source
}
fn until_hand_choice(g: &mut Game, player: Player) {
    for _ in 0..80 {
        if matches!(g.state.pending, Pending::HandCardChoice { .. }) { return; }
        if g.legal_actions().contains(&Action::ChooseTarget { target: Target::Player(player.other()) }) {
            act(g, Action::ChooseTarget { target: Target::Player(player.other()) });
        } else {
            choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets));
        }
    }
    panic!("hand choice not reached: {:?}", g.state.pending);
}

#[test]
fn null_reveals_hand_requires_nonland_choice_and_casts_at_threshold() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let card = put(&mut g, player.other(), "Spectral Sailor", Zone::Hand);
        let land = put(&mut g, player.other(), "Forest", Zone::Hand);
        let source = cast_null(&mut g, player);
        until_hand_choice(&mut g, player);
        for viewer in [player, player.other()] {
            let view = g.observe(viewer);
            assert!(view.known_hand.contains(&g.object_ref(card)));
            assert!(view.known_hand.contains(&g.object_ref(land)));
        }
        reject(&mut g, Action::ChooseHandCard { object: None });
        let bad = g.object_ref(land);
        reject(&mut g, Action::ChooseHandCard { object: Some(bad) });
        let before = g.snapshot();
        for _ in 0..2 {
            let chosen = g.object_ref(card);
            act(&mut g, Action::ChooseHandCard { object: Some(chosen) });
            assert_eq!(g.object_view(card).zone, Zone::Exile);
            assert_eq!(g.state.objects[source as usize].controller, player);
            for _ in 0..6 { put(&mut g, player, "Forest", Zone::Graveyard); }
            assert!(!g.can_play_exiled(player, card));
            put(&mut g, player, "Forest", Zone::Graveyard);
            assert!(g.can_play_exiled(player, card));
            let purpose = g.spell_mana_purpose_for_card(player, card, g.state.objects[card as usize].card);
            assert!(cast_origins::spends_any_type(&g, player, purpose));
            g.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
            act(&mut g, Action::AnnounceExile { card });
            pay(&mut g);
            assert_eq!(g.state.players[player.index()].mana, [0; 6]);
            g.restore(&before).unwrap();
        }
    }
}

#[test]
fn null_source_departure_does_not_cancel_reveal_or_exile() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let card = put(&mut g, player.other(), "Spectral Sailor", Zone::Hand);
        let source = cast_null(&mut g, player);
        until_hand_choice(&mut g, player);
        let old_source = g.object_ref(source);
        g.move_object(source, Zone::Graveyard);
        let chosen = g.object_ref(card);
        act(&mut g, Action::ChooseHandCard { object: Some(chosen) });
        assert_eq!(g.object_view(card).zone, Zone::Exile);
        for _ in 0..7 { put(&mut g, player, "Forest", Zone::Graveyard); }
        assert!(!g.can_play_exiled(player, card));
        g.move_object(source, Zone::Battlefield);
        assert_ne!(g.object_ref(source), old_source);
        assert!(!g.can_play_exiled(player, card));
    }
}

#[test]
fn null_link_follows_controller_threshold_and_ability_removal() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let card = put(&mut g, player.other(), "Spectral Sailor", Zone::Hand);
        let source = cast_null(&mut g, player);
        until_hand_choice(&mut g, player);
        let chosen = g.object_ref(card);
        act(&mut g, Action::ChooseHandCard { object: Some(chosen) });
        for p in [player, player.other()] { for _ in 0..7 { put(&mut g, p, "Forest", Zone::Graveyard); } }
        let source_ref = g.object_ref(source);
        let saved = g.snapshot();
        assert!(g.can_play_exiled(player, card));
        g.change_controller(source_ref, player.other());
        assert!(!g.can_play_exiled(player, card));
        assert!(g.can_play_exiled(player.other(), card));
        g.restore(&saved).unwrap();
        let mut item = g.activation_item(player, source, 0);
        item.targets = vec![BoundTarget { group: 0, target: Target::Card(source), object: Some(source_ref) }];
        g.remove_target_abilities_until_end_of_turn(&item);
        assert!(!g.can_play_exiled(player, card));
        g.restore(&saved).unwrap();
        assert!(g.can_play_exiled(player, card));
    }
}

#[test]
fn null_does_not_trigger_when_not_cast_and_reveals_land_only_hand() {
    for player in [Player::P0, Player::P1] {
        let mut g = scenario(player);
        let source = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.objects[source as usize].card = Card::from_name("Null Summoner").unwrap();
        let land = put(&mut g, player.other(), "Forest", Zone::Hand);
        g.move_object(source, Zone::Battlefield);
        settle(&mut g);
        assert!(!g.observe(player).known_hand.contains(&g.object_ref(land)));
        g.move_object(source, Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: source });
        pay(&mut g);
        for _ in 0..80 {
            if g.state.pending == Pending::Priority && g.state.stack.is_empty()
                && g.state.pending_triggers.is_empty() && g.state.resolving.is_none() { break; }
            if g.legal_actions().contains(&Action::ChooseTarget { target: Target::Player(player.other()) }) {
                act(&mut g, Action::ChooseTarget { target: Target::Player(player.other()) });
            } else {
                choose(&mut g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets));
            }
        }
        assert!(g.observe(player).known_hand.contains(&g.object_ref(land)));
        assert_eq!(g.object_view(land).zone, Zone::Hand);
        assert!(g.state.linked_choices.as_ref().is_none_or(|data| data.null_cast_links.is_empty()));
    }
}

#[test]
fn null_mana_permission_tracks_live_and_projected_payment_world() {
    for player in [Player::P0, Player::P1] {
        for loss in 0..4 {
            let mut g = scenario(player);
            let card = put(&mut g, player.other(), "Spectral Sailor", Zone::Hand);
            let source = cast_null(&mut g, player);
            until_hand_choice(&mut g, player);
            let chosen = g.object_ref(card);
            act(&mut g, Action::ChooseHandCard { object: Some(chosen) });
            let graves: Vec<_> = (0..7).map(|_| put(&mut g, player, "Forest", Zone::Graveyard)).collect();
            g.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
            act(&mut g, Action::AnnounceExile { card });
            let saved = g.snapshot();
            let source_ref = g.object_ref(source);
            for _ in 0..2 {
                match loss {
                    0 => { g.move_object(source, Zone::Graveyard); }
                    1 => { g.move_object(graves[0], Zone::Hand); }
                    2 => g.change_controller(source_ref, player.other()),
                    _ => {
                        let mut item = g.activation_item(player, source, 0);
                        item.targets = vec![BoundTarget { group: 0, target: Target::Card(source), object: Some(source_ref) }];
                        g.remove_target_abilities_until_end_of_turn(&item);
                    }
                }
                let item = g.state.announcing.as_ref().unwrap().item.clone();
                let purpose = g.purpose_for_item(&item);
                let goal = FundingGoal { actor: player, required: [0, 1, 0, 0, 0, 0], generic: 0, reserve: None, source: None };
                assert!(!cast_origins::spends_any_type(&g, player, purpose));
                assert!(!g.can_fund_for(goal.clone(), purpose));
                // A separate live rule may still authorize red mana.
                put(&mut g, player, "Vizier of the Menagerie", Zone::Battlefield);
                assert!(cast_origins::spends_any_type(&g, player, purpose));
                assert!(g.can_fund_for(goal, purpose));
                g.restore(&saved).unwrap();
                assert_eq!(g.snapshot(), saved);
            }
        }
    }
}
