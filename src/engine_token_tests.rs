use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
fn factory(name: &str) -> Card {
    Card::catalogue()
        .find(|c| c.token_info().is_some_and(|info| info.name == name))
        .unwrap()
}
fn tokens(game: &Game) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Battlefield && o.card.is_token())
        .map(|(id, _)| id as CardId)
        .collect()
}
fn enter(game: &mut Game, name: &str) -> CardId {
    let id = spawn(game, Player::P0, card(name), Zone::Hand);
    game.move_object(id, Zone::Battlefield);
    game.state_based_actions();
    game.prepare_triggers();
    id
}

#[test]
fn token_spell_creates_distinct_summoning_sick_objects_and_replays() {
    let mut game = scenario();
    let fodder = spawn(&mut game, Player::P0, card("Dragon Fodder"), Zone::Hand);
    game.state.players[0].mana = [1, 0, 0, 1, 0, 0];
    cast(&mut game, fodder, None);
    let before = game.snapshot();
    pass_pair(&mut game);
    let created = tokens(&game);
    assert_eq!(created.len(), 2);
    assert_ne!(game.object_ref(created[0]), game.object_ref(created[1]));
    for id in created {
        let token = game.object_view(id);
        assert_eq!(token.token.unwrap().name, "Goblin Token");
        assert_eq!(token.token.unwrap().colors, 8);
        assert_eq!((token.power, token.toughness), (1, 1));
        assert!(token.summoning_sick && !token.tapped);
        assert_eq!(token.owner, Player::P0);
    }
    assert_eq!(game.state.objects[fodder as usize].zone, Zone::Graveyard);
    let after = game.snapshot();
    game.restore(&before).unwrap();
    pass_pair(&mut game);
    assert_eq!(game.snapshot(), after);
}

#[test]
fn simultaneous_creation_queues_one_entry_trigger_per_token_and_preserves_choices() {
    let mut game = scenario();
    spawn(
        &mut game,
        Player::P0,
        card("Hinterland Sanctifier"),
        Zone::Battlefield,
    );
    game.create_tokens(Player::P0, factory("Dog Token"), 4, false);
    assert_eq!(tokens(&game).len(), 4);
    assert_eq!(game.state.pending_triggers.len(), 4);
    assert_eq!(game.state.players[0].life, 20);
    game.prepare_triggers();
    let before = game.snapshot();
    while let Some(action) = game
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
    {
        act(&mut game, action);
    }
    for _ in 0..4 {
        pass_pair(&mut game);
    }
    assert_eq!(game.state.players[0].life, 24);
    let after = game.snapshot();
    game.restore(&before).unwrap();
    while let Some(action) = game
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PutTrigger { .. }))
    {
        act(&mut game, action);
    }
    for _ in 0..4 {
        pass_pair(&mut game);
    }
    assert_eq!(game.snapshot(), after);
}

#[test]
fn token_cannot_move_again_after_leaving_and_ceases_with_no_visible_tombstone() {
    for destination in [Zone::Hand, Zone::Graveyard, Zone::Library] {
        let mut game = scenario();
        game.create_tokens(Player::P0, factory("Soldier Token"), 1, false);
        let id = tokens(&game)[0];
        let before = game.object_ref(id);
        let event = game.move_object(id, destination);
        assert!(event.moved);
        assert_eq!(event.before.reference, before);
        assert_eq!(event.before.token.unwrap().name, "Soldier Token");
        let departed = game.object_ref(id);
        assert_ne!(departed, before);
        let returned = game.move_object(id, Zone::Battlefield);
        assert!(!returned.moved);
        assert_eq!(game.object_ref(id), departed);
        assert_eq!(game.state.objects[id as usize].zone, destination);
        assert_eq!(game.hand_count(Player::P0), 0);
        assert!(!game.target_matches(
            TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            Player::P0,
            before,
            Target::Card(id)
        ));
        game.state_based_actions();
        assert_eq!(game.state.objects[id as usize].zone, Zone::Outside);
        assert_ne!(game.object_ref(id), departed);
        assert!(!game.state.players[0].library.contains(&id));
        for viewer in [Player::P0, Player::P1] {
            assert!(!game.observe(viewer).objects.iter().any(|o| o.id == id));
        }
        assert_eq!(game.state.free_tokens, vec![id]);
        game.state_based_actions();
        assert_eq!(game.state.free_tokens, vec![id]);
    }
}

#[test]
fn reclaimed_slots_never_alias_old_targets_and_restore_the_free_list() {
    let mut game = scenario();
    let token = factory("Soldier Token");
    game.create_tokens(Player::P0, token, 1, false);
    let id = tokens(&game)[0];
    let growth = spawn(&mut game, Player::P0, Card::GiantGrowth, Zone::Hand);
    game.state.players[0].mana[4] = 1;
    cast(&mut game, growth, Some(Target::Card(id)));
    let original = game.object_ref(id);
    game.move_object(id, Zone::Graveyard);
    game.state_based_actions();
    let before_reuse = game.snapshot();
    let slots = game.state.objects.len();
    game.create_tokens(Player::P1, factory("Dragon Token"), 1, true);
    assert_eq!(tokens(&game), vec![id]);
    assert_eq!(game.state.objects.len(), slots);
    assert_ne!(game.object_ref(id), original);
    let base_power = game.object_view(id).power;
    pass_pair(&mut game);
    assert_eq!(game.object_view(id).power, base_power);
    assert!(game.object_view(id).tapped);
    let after = game.snapshot();
    game.restore(&before_reuse).unwrap();
    game.create_tokens(Player::P1, factory("Dragon Token"), 1, true);
    pass_pair(&mut game);
    assert_eq!(game.snapshot(), after);
    for _ in 0..100 {
        game.move_object(id, Zone::Graveyard);
        game.state_based_actions();
        game.create_tokens(Player::P1, token, 1, false);
    }
    assert_eq!(game.state.objects.len(), slots);
}

#[test]
fn death_ability_creates_tapped_artifact_token_after_source_leaves() {
    let mut game = scenario();
    let automaton = spawn(
        &mut game,
        Player::P0,
        card("Machinesmith Automaton"),
        Zone::Battlefield,
    );
    let monoist = spawn(
        &mut game,
        Player::P0,
        card("Gravpack Monoist"),
        Zone::Battlefield,
    );
    game.move_object(monoist, Zone::Graveyard);
    game.prepare_triggers();
    let before = game.snapshot();
    pass_pair(&mut game);
    let id = tokens(&game)[0];
    assert_eq!(game.object_view(id).token.unwrap().name, "Robot Token");
    assert!(game.object_view(id).tapped);
    assert!(game.state.objects[id as usize].card.kind_mask() & 16 != 0);
    assert_eq!(game.state.stack.len(), 1);
    pass_pair(&mut game);
    assert_eq!(game.object_view(automaton).counters.plus_one, 1);
    let after = game.snapshot();
    game.restore(&before).unwrap();
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.snapshot(), after);
}

#[test]
fn named_and_multicolor_factories_preserve_characteristics_and_block_restrictions() {
    let mut game = scenario();
    enter(&mut game, "Heartstring Puller");
    pass_pair(&mut game);
    let cadet = game.object_view(tokens(&game)[0]);
    assert_eq!(cadet.token.unwrap().name, "Cadet");
    assert_eq!(cadet.token.unwrap().subtypes, "Wizard Soldier");
    assert_eq!(cadet.token.unwrap().colors, 0);
    enter(&mut game, "Eager Glyphmage");
    pass_pair(&mut game);
    let inkling = tokens(&game)
        .into_iter()
        .map(|id| game.object_view(id))
        .find(|o| o.token.unwrap().name == "Inkling Token")
        .unwrap();
    assert_eq!(inkling.token.unwrap().colors, 5);
    assert!(inkling.keywords.contains(Keywords::FLYING));
    let rat = factory("Rat Token");
    game.create_tokens(Player::P1, rat, 1, false);
    let rat_id = *tokens(&game).last().unwrap();
    let attacker = spawn(&mut game, Player::P0, Card::HillGiant, Zone::Battlefield);
    game.state.attackers = vec![Attacker {
        defender: Some(CombatDefender::Player(Player::P1)),
        card: attacker,
        blocked: false,
    }];
    game.state.pending = Pending::Blockers;
    game.state.step = Step::Blockers;
    assert_eq!(game.actor(), Some(Player::P1));
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|action| matches!(action, Action::Block { blocker, .. } if *blocker == rat_id))
    );
}

#[test]
fn lethal_token_damage_ceases_and_reclaims_in_the_same_sba_fixed_point() {
    let mut game = scenario();
    game.create_tokens(Player::P0, factory("Soldier Token"), 2, false);
    let created = tokens(&game);
    for &id in &created {
        game.state.objects[id as usize].damage = 1;
    }
    let before = game.snapshot();
    game.state_based_actions();
    assert!(tokens(&game).is_empty());
    assert_eq!(game.state.free_tokens.len(), 2);
    assert!(
        created
            .iter()
            .all(|&id| game.state.objects[id as usize].zone == Zone::Outside)
    );
    let after = game.snapshot();
    game.restore(&before).unwrap();
    game.state_based_actions();
    assert_eq!(game.snapshot(), after);
}
