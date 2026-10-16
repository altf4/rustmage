use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for player in &mut g.state.players {
        player.mana = [20; 6];
    }
    g
}
fn known(g: &Game, id: CardId) {
    let reference = g.object_ref(id);
    assert_eq!(g.object_view(id).zone, Zone::Hand);
    assert_eq!(
        g.state
            .known_hand
            .iter()
            .filter(|&&r| r == reference)
            .count(),
        1
    );
    for viewer in [Player::P0, Player::P1] {
        let view = g.observe(viewer);
        assert!(view.known_hand.contains(&reference));
        assert!(
            view.objects
                .iter()
                .any(|o| o.reference == reference && o.zone == Zone::Hand)
        );
    }
}
fn hidden(g: &Game, id: CardId) {
    let owner = g.object_view(id).owner;
    assert!(!g.state.known_hand.iter().any(|r| r.card == id));
    assert!(!g.observe(owner.other()).objects.iter().any(|o| o.id == id));
    assert!(g.observe(owner).objects.iter().any(|o| o.id == id));
}
fn replay(g: &mut Game, action: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    let actor = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, action.clone());
    act(g, action.clone());
    let expected = g.snapshot();
    assert_eq!(fork.snapshot(), expected);
    g.restore(&saved).unwrap();
    assert_eq!(g.apply_index(actor, revision, 0), Err(Error::StaleRevision));
    act(g, action);
    assert_eq!(g.snapshot(), expected);
}

#[test]
fn targeted_bounce_preserves_public_new_hand_identity_until_real_recast() {
    for p in [Player::P0, Player::P1] {
        for caster in [p, p.other()] {
            let mut g = setup(p);
            let creature = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let private = named(&mut g, p, "Bear Cub", Zone::Hand);
            let bounce = named(&mut g, caster, "Unsummon", Zone::Hand);
            let original = g.object_ref(creature);
            if caster != p {
                act(&mut g, Action::Pass);
            }
            cast(&mut g, bounce, Some(Target::Card(creature)));
            act(&mut g, Action::Pass);
            replay(&mut g, Action::Pass);
            known(&g, creature);
            hidden(&g, private);
            let hand = g.object_ref(creature);
            assert_ne!(hand, original);
            cast(&mut g, creature, None);
            assert!(!g.state.known_hand.contains(&hand));
            pass_pair(&mut g);
            assert_eq!(g.object_view(creature).zone, Zone::Battlefield);
            assert_ne!(g.object_ref(creature), hand);
            hidden(&g, private);
        }
    }
}

#[test]
fn targeted_graveyard_recovery_is_public_for_both_owners_and_replays() {
    for p in [Player::P0, Player::P1] {
        for source in ["Bonebind Orator", "Theoretical Necromancer"] {
            let mut g = setup(p);
            let id = named(&mut g, p, source, Zone::Graveyard);
            let recovered = named(&mut g, p, "Bear Cub", Zone::Graveyard);
            let private = named(&mut g, p, "Bear Cub", Zone::Hand);
            let original = g.object_ref(recovered);
            let action = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(a,
                Action::Activate { card, target: Some(Target::Card(t)), .. }
                if *card == id && *t == recovered)
                })
                .unwrap();
            replay(&mut g, action);
            assert_eq!(g.object_view(id).zone, Zone::Exile);
            act(&mut g, Action::Pass);
            replay(&mut g, Action::Pass);
            known(&g, recovered);
            assert_ne!(g.object_ref(recovered), original);
            hidden(&g, private);
            assert_eq!(g.state.known_hand.len(), 1);
        }
    }
}

#[test]
fn shared_public_zone_event_batches_keep_exact_refs_without_duplicate_or_stale_facts() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ids: Vec<_> = [Zone::Battlefield, Zone::Graveyard, Zone::Stack, Zone::Exile]
            .into_iter()
            .map(|z| named(&mut g, p, "Bear Cub", z))
            .collect();
        let old: Vec<_> = ids.iter().map(|&id| g.object_ref(id)).collect();
        let saved = g.snapshot();
        let perform = |g: &mut Game| {
            let events = g.move_objects_raw(&ids, Zone::Hand);
            // All objects have moved before any observers are queued.
            g.finish_zone_events(&events, &[]);
            g.finish_zone_events(&events, &[]);
            for &id in &ids {
                known(g, id);
            }
            assert_eq!(g.state.known_hand.len(), ids.len());
            events
        };
        let mut fork = g.clone();
        perform(&mut fork);
        let events = perform(&mut g);
        assert_eq!(g.snapshot(), fork.snapshot());
        let expected = g.snapshot();
        g.restore(&saved).unwrap();
        perform(&mut g);
        assert_eq!(g.snapshot(), expected);
        assert!(old.iter().all(|r| !g.state.known_hand.contains(r)));
        for &id in &ids {
            g.move_object(id, Zone::Library);
            g.draw(p);
            hidden(&g, id);
        }
        // Replaying an old public event cannot expose a later private draw of
        // the same physical card, even when it is currently back in hand.
        g.finish_zone_events(&events, &[]);
        assert!(g.state.known_hand.is_empty());
        for &id in &ids {
            hidden(&g, id);
        }
    }
}

#[test]
fn staged_public_and_explicitly_revealed_library_entries_deduplicate_hand_knowledge() {
    let mut g = setup(Player::P0);
    let public = named(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
    let revealed = named(&mut g, Player::P1, "Bear Cub", Zone::Library);
    let private = named(&mut g, Player::P1, "Bear Cub", Zone::Library);
    let moves = [public, revealed, private]
        .into_iter()
        .map(|id| {
            let mut movement = g.entry_move(id, Zone::Hand, false);
            movement.reveal_hand = id != private;
            movement
        })
        .collect();
    g.stage_entries(moves, None);
    known(&g, public);
    known(&g, revealed);
    hidden(&g, private);
    assert_eq!(g.state.known_hand.len(), 2);
}

#[test]
fn private_draws_and_unrelated_hands_stay_hidden_beside_public_returns() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let original = named(&mut g, p, "Bear Cub", Zone::Hand);
        let drawn = named(&mut g, p, "Bear Cub", Zone::Library);
        let returned = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        g.move_object(returned, Zone::Hand);
        g.draw(p);
        known(&g, returned);
        hidden(&g, drawn);
        hidden(&g, original);
        let hand = g.object_ref(returned);
        g.move_object(returned, Zone::Graveyard);
        assert!(!g.state.known_hand.contains(&hand));
        g.move_object(returned, Zone::Library);
        g.draw(p);
        hidden(&g, returned);
    }
}

#[test]
fn bounced_tokens_cease_without_stale_hand_facts_and_cannot_move_again() {
    let mut g = setup(Player::P0);
    let token = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|i| i.name == "Goblin Token"))
        .unwrap();
    let id = spawn(&mut g, Player::P0, token, Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Unsummon", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(id)));
    pass_pair(&mut g);
    assert_eq!(g.object_view(id).zone, Zone::Outside);
    assert!(g.state.known_hand.is_empty());
    let event = g.move_object(id, Zone::Hand);
    assert!(!event.moved);
    assert!(g.state.known_hand.is_empty());
}

#[test]
fn failed_bounce_capacity_rolls_back_existing_public_knowledge_and_resolution() {
    let mut g = setup(Player::P0);
    let known_id = named(&mut g, Player::P0, "Bear Cub", Zone::Graveyard);
    g.move_object(known_id, Zone::Hand);
    let target = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    g.state.objects[target as usize].generation = u64::MAX;
    let bounce = named(&mut g, Player::P0, "Unsummon", Zone::Hand);
    cast(&mut g, bounce, Some(Target::Card(target)));
    act(&mut g, Action::Pass);
    let saved = g.snapshot();
    let revision = g.revision();
    let actor = g.actor().unwrap();
    for _ in 0..2 {
        assert!(matches!(
            g.apply(actor, Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), revision);
        known(&g, known_id);
    }
}
