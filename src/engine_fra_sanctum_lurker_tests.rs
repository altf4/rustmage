//! Gated Sanctum Lurker, its state-based-action exception, and loyalty grant.
use super::*;

fn lurker(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Bear Cub", Zone::Battlefield);
    g.state.objects[id as usize].card = Card::from_name("Sanctum Lurker").unwrap();
    g.register_battlefield_sources(id);
    id
}

#[test]
fn lurker_survives_zero_loyalty_only_for_its_controller_and_rechecks() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = lurker(&mut g, p);
        let own = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        let enemy = put(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.state.objects[own as usize].counters.loyalty = 0;
        g.state.objects[enemy as usize].counters.loyalty = 0;
        let saved = g.snapshot();
        for _ in 0..2 {
            g.state_based_actions();
            assert_eq!(g.object_view(own).zone, Zone::Battlefield);
            assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
            g.move_object(provider, Zone::Exile);
            g.state_based_actions();
            assert_eq!(g.object_view(own).zone, Zone::Graveyard);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn lurker_grant_raises_loyalty_and_uses_walker_as_damage_source() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = lurker(&mut g, p);
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 0;
        let source = g.object_ref(walker);
        let grantor = g.object_ref(provider);
        let action = Action::AnnouncePlaneswalkerGrant {
            source,
            grantor,
            ability: 0,
        };
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, action.clone());
            pay(&mut g);
            assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 2);
            assert!(!g.legal_actions().contains(&action));
            g.move_object(provider, Zone::Exile);
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].life, 21);
            assert_eq!(g.state.players[p.other().index()].life, 19);
            assert!(!g.legal_actions().iter().any(|a| matches!(a,
                Action::AnnounceActivate { card, .. } if *card == walker)));
            assert!(g.state.objects[walker as usize].dealt_damage_this_turn);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn lurker_copy_and_blanking_change_zero_loyalty_exception() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = lurker(&mut g, p);
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 0;
        let saved = g.snapshot();
        let mut blank = g.activation_item(p, source, 0);
        blank.targets = vec![BoundTarget {
            group: 0,
            target: Target::Card(source),
            object: Some(g.object_ref(source)),
        }];
        g.remove_target_abilities_until_end_of_turn(&blank);
        g.state_based_actions();
        assert_eq!(g.object_view(walker).zone, Zone::Graveyard);
        g.restore(&saved).unwrap();
        let reference = g.object_ref(source);
        g.change_controller(reference, p.other());
        g.state_based_actions();
        assert_eq!(g.object_view(walker).zone, Zone::Graveyard);
        g.restore(&saved).unwrap();
        let copy = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let copy_ref = g.object_ref(copy);
        g.apply_temporary_copy(
            copy_ref,
            CopyRecipe::new(Card::from_name("Sanctum Lurker").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        g.move_object(source, Zone::Exile);
        g.state_based_actions();
        assert_eq!(g.object_view(walker).zone, Zone::Battlefield);
    }
}

#[test]
fn lurker_paid_cast_empowers_a_new_jace_and_replays() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let card = put(&mut g, p, "Sanctum Lurker", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
        let saved = g.snapshot();
        for _ in 0..2 {
            act(&mut g, Action::AnnounceCast { card });
            pay(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            settle(&mut g);
            assert_eq!(g.object_view(card).zone, Zone::Battlefield);
            let jace = g
                .state
                .objects
                .iter()
                .enumerate()
                .find_map(|(i, o)| {
                    (o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.is_token()
                        && g.has_subtype(i as CardId, "Jace"))
                    .then_some(i as CardId)
                })
                .unwrap();
            assert_eq!(g.counter_count(jace, CounterKind::Loyalty), 1);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn lurker_lethal_damage_dies_first_then_zero_loyalty_walker_dies() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = lurker(&mut g, p);
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 0;
        g.state.objects[provider as usize].damage = 2;
        let saved = g.snapshot();
        for _ in 0..2 {
            g.state_based_actions();
            assert_eq!(g.object_view(provider).zone, Zone::Graveyard);
            assert_eq!(g.object_view(walker).zone, Zone::Graveyard);
            g.restore(&saved).unwrap();
        }
    }
}

#[test]
fn lurker_grant_follows_controller_and_ability_removal() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let provider = lurker(&mut g, p);
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Battlefield);
        g.state.objects[walker as usize].counters.loyalty = 3;
        let source = g.object_ref(walker);
        let grantor = g.object_ref(provider);
        let action = Action::AnnouncePlaneswalkerGrant {
            source,
            grantor,
            ability: 0,
        };
        assert!(g.legal_actions().contains(&action));
        let saved = g.snapshot();
        let mut blank = g.activation_item(p, provider, 0);
        blank.targets = vec![BoundTarget {
            group: 0,
            target: Target::Card(provider),
            object: Some(grantor),
        }];
        g.remove_target_abilities_until_end_of_turn(&blank);
        assert!(!g.legal_actions().contains(&action));
        g.restore(&saved).unwrap();
        g.change_controller(grantor, p.other());
        assert!(!g.legal_actions().contains(&action));
        g.restore(&saved).unwrap();
        assert!(g.legal_actions().contains(&action));
    }
}
