use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn enchant(g: &mut Game, name: &str, target: CardId) -> CardId {
    let id = named(g, Player::P0, name, Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    g.prepare_triggers();
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    id
}
fn exile(g: &mut Game, name: &str, target: CardId) -> CardId {
    let source = enchant(g, name, target);
    pass_pair(g);
    source
}

#[test]
fn linked_exile_is_public_owned_and_returns_fresh_incarnation_with_etb() {
    let mut g = scenario();
    let target = named(&mut g, Player::P1, "Helpful Hunter", Zone::Battlefield);
    g.state.objects[target as usize].counters.plus_one = 2;
    g.state.objects[target as usize].damage = 1;
    let old = g.object_ref(target);
    let source = exile(&mut g, "Banishing Light", target);
    assert_eq!(g.state.objects[target as usize].zone, Zone::Exile);
    assert_ne!(g.object_ref(target), old);
    assert_eq!(g.state.exile_links.len(), 1);
    let saved = g.snapshot();
    for viewer in [Player::P0, Player::P1] {
        let view = g.observe(viewer);
        assert_eq!(view.schema, 45);
        assert_eq!(view.exile_links, g.state.exile_links);
        assert!(
            view.objects
                .iter()
                .any(|o| o.id == target && o.zone == Zone::Exile)
        );
    }
    let branch = g.clone();
    assert_eq!(branch.state, g.state);
    g.move_object(source, Zone::Hand);
    assert_eq!(g.state.objects[target as usize].zone, Zone::Battlefield);
    assert!(g.state.objects[target as usize].summoning_sick);
    assert_eq!(
        g.state.objects[target as usize].counters,
        Counters::default()
    );
    assert_eq!(g.state.objects[target as usize].damage, 0);
    assert!(g.state.exile_links.is_empty());
    assert_eq!(g.state.pending_triggers.len(), 1);
    let result = g.state.clone();
    g.restore(&saved).unwrap();
    g.move_object(source, Zone::Hand);
    assert_eq!(g.state, result);
}

#[test]
fn source_leaving_and_returning_before_resolution_does_not_exile_old_target() {
    for reenter in [false, true] {
        let mut g = scenario();
        let target = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
        let other = named(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
        let source = enchant(&mut g, "Banishing Light", target);
        let old = g.object_ref(source);
        g.move_object(source, Zone::Hand);
        if reenter {
            g.move_object(source, Zone::Battlefield);
            g.prepare_triggers();
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(other),
                },
            );
            pass_pair(&mut g);
            assert_eq!(g.state.objects[other as usize].zone, Zone::Exile);
        }
        pass_pair(&mut g);
        assert_eq!(g.state.objects[target as usize].zone, Zone::Battlefield);
        assert!(g.state.exile_links.iter().all(|link| link.source != old));
        assert_eq!(g.state.exile_links.len(), usize::from(reenter));
    }
}

#[test]
fn simultaneous_sources_return_cards_together_before_entry_trigger_matching() {
    let mut g = scenario();
    let first = named(&mut g, Player::P1, "Dazzling Angel", Zone::Battlefield);
    let second = named(&mut g, Player::P1, "Dazzling Angel", Zone::Battlefield);
    let one = exile(&mut g, "Banishing Light", first);
    let two = exile(&mut g, "Banishing Light", second);
    assert!(g.state.pending_triggers.is_empty());
    let saved = g.snapshot();
    let events = g.move_objects_raw(&[one, two], Zone::Graveyard);
    g.finish_zone_events(&events, &[]);
    assert_eq!(g.state.objects[first as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.objects[second as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.pending_triggers.len(), 2);
    assert!(g.state.exile_links.is_empty());
    let result = g.state.clone();
    g.restore(&saved).unwrap();
    let events = g.move_objects_raw(&[one, two], Zone::Graveyard);
    g.finish_zone_events(&events, &[]);
    assert_eq!(g.state, result);
}

#[test]
fn tokens_cease_in_exile_and_never_return_or_retain_links() {
    let mut g = scenario();
    let factory = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Elf Warrior"))
        .unwrap();
    g.create_tokens(Player::P1, factory, 1, false);
    let token = (g.state.objects.len() - 1) as CardId;
    let source = exile(&mut g, "Banishing Light", token);
    assert_eq!(g.state.objects[token as usize].zone, Zone::Outside);
    assert!(g.state.exile_links.is_empty());
    g.move_object(source, Zone::Hand);
    assert_eq!(g.state.objects[token as usize].zone, Zone::Outside);
    g.create_tokens(Player::P1, factory, 1, false);
    assert_eq!(g.state.objects[token as usize].zone, Zone::Battlefield);
    assert!(g.state.exile_links.is_empty());
}

#[test]
fn linked_tapped_and_mana_value_filters_revalidate_and_apply_to_tokens() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    g.state.objects[bear as usize].tapped = true;
    let source = enchant(&mut g, "Super Villain Lockup", bear);
    g.state.objects[bear as usize].untap();
    pass_pair(&mut g);
    assert_eq!(g.state.objects[bear as usize].zone, Zone::Battlefield);
    assert!(g.state.exile_links.is_empty());
    let casket = named(&mut g, Player::P0, "Glass Casket", Zone::Hand);
    g.move_object(casket, Zone::Battlefield);
    g.prepare_triggers();
    let giant = named(&mut g, Player::P1, "Hill Giant", Zone::Battlefield);
    let factory = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Elf Warrior"))
        .unwrap();
    g.create_tokens(Player::P1, factory, 1, false);
    let token = (g.state.objects.len() - 1) as CardId;
    assert!(g.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Card(token)
    }));
    assert_eq!(g.state.objects[token as usize].card.fixed_mana_value(), 0);
    assert!(!g.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Card(giant)
    }));
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    pass_pair(&mut g);
    assert_eq!(g.state.objects[bear as usize].zone, Zone::Exile);
    g.move_object(casket, Zone::Graveyard);
    assert_eq!(g.state.objects[bear as usize].zone, Zone::Battlefield);
    assert_eq!(g.state.objects[source as usize].zone, Zone::Battlefield);
    let down = named(&mut g, Player::P0, "Epic Downfall", Zone::Hand);
    g.state.players[0].mana = [0, 0, 2, 0, 0, 0];
    let actions = g.legal_actions();
    assert!(actions.iter().any(|a|matches!(a, Action::Cast { card, target: Some(Target::Card(id)), .. } if *card==down && *id==giant)));
    assert!(!actions.iter().any(|a|matches!(a, Action::Cast { card, target: Some(Target::Card(id)), .. } if *card==down && (*id==token || *id==bear))));
}

#[test]
fn graveyard_exile_has_no_death_triggers_and_soul_guide_lantern_sacrifices_normally() {
    let mut g = scenario();
    let own = named(&mut g, Player::P0, "Grizzly Bears", Zone::Graveyard);
    let card = named(&mut g, Player::P1, "Agents of HYDRA", Zone::Graveyard);
    let next = named(&mut g, Player::P1, "Forest", Zone::Graveyard);
    let source = named(&mut g, Player::P0, "Soul-Guide Lantern", Zone::Hand);
    g.move_object(source, Zone::Battlefield);
    g.prepare_triggers();
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
    pass_pair(&mut g);
    assert_eq!(g.state.objects[card as usize].zone, Zone::Exile);
    assert!(g.state.pending_triggers.is_empty());
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Activate{card,ability:0,..} if *card==source))
        .unwrap();
    act(&mut g, action);
    assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
    pass_pair(&mut g);
    assert_eq!(g.state.objects[next as usize].zone, Zone::Exile);
    assert_eq!(g.state.objects[own as usize].zone, Zone::Graveyard);
}

#[test]
fn direct_exile_ignores_indestructible_and_does_not_count_as_dying() {
    let mut g = scenario();
    let target = named(&mut g, Player::P1, "Agents of HYDRA", Zone::Battlefield);
    g.state.objects[target as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    let spell = named(&mut g, Player::P0, "Wander Off", Zone::Hand);
    let (mut mana, generic) = g.state.objects[spell as usize].card.cost();
    mana[5] += generic;
    g.state.players[0].mana = mana;
    cast(&mut g, spell, Some(Target::Card(target)));
    pass_pair(&mut g);
    assert_eq!(g.state.objects[target as usize].zone, Zone::Exile);
    assert!(g.state.stack.is_empty());
    assert!(g.state.pending_triggers.is_empty());
}

#[test]
fn source_sba_death_returns_creature_before_next_priority_and_replays() {
    let mut g = scenario();
    let target = named(&mut g, Player::P1, "Helpful Hunter", Zone::Battlefield);
    g.state.objects[target as usize].tapped = true;
    let source = exile(&mut g, "Henchbots", target);
    g.state.objects[source as usize].damage = g.object_view(source).toughness;
    let saved = g.snapshot();
    g.state_based_actions();
    assert_eq!(g.state.objects[source as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.objects[target as usize].zone, Zone::Battlefield);
    assert!(g.state.exile_links.is_empty());
    assert_eq!(g.state.pending_triggers.len(), 1);
    let result = g.state.clone();
    g.restore(&saved).unwrap();
    g.state_based_actions();
    assert_eq!(g.state, result);
}

#[test]
fn departing_exiled_incarnation_cannot_be_returned_by_stale_link() {
    let mut g = scenario();
    let target = named(&mut g, Player::P1, "Grizzly Bears", Zone::Battlefield);
    let source = exile(&mut g, "Banishing Light", target);
    let old = g.object_ref(target);
    g.move_object(target, Zone::Hand);
    assert!(g.state.exile_links.is_empty());
    g.move_object(target, Zone::Exile);
    assert_ne!(g.object_ref(target), old);
    g.move_object(source, Zone::Hand);
    assert_eq!(g.state.objects[target as usize].zone, Zone::Exile);
}

#[test]
fn token_cannot_return_even_when_source_leaves_before_cessation_sba() {
    let mut g = scenario();
    let factory = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Elf Warrior"))
        .unwrap();
    g.create_tokens(Player::P1, factory, 1, false);
    let token = (g.state.objects.len() - 1) as CardId;
    let source = enchant(&mut g, "Banishing Light", token);
    // Kernel boundary: resolve without the caller's subsequent SBA check.
    g.resolve_top();
    assert_eq!(g.state.objects[token as usize].zone, Zone::Exile);
    g.move_object(source, Zone::Hand);
    assert_eq!(g.state.objects[token as usize].zone, Zone::Exile);
    assert!(g.state.exile_links.is_empty());
    g.state_based_actions();
    assert_eq!(g.state.objects[token as usize].zone, Zone::Outside);
}
