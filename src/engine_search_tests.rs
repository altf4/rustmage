use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;

fn named(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}
fn activate(game: &mut Game, id: CardId, ability: u16) {
    let action = game
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(a,
        Action::Activate { card, ability: index, .. } if *card == id && *index == ability)
        })
        .unwrap();
    act(game, action);
}
fn enter_search(game: &mut Game, name: &str, ability: u16, mana: Mana) -> CardId {
    let source = named(game, Player::P0, name, Zone::Battlefield);
    game.state.players[0].mana = mana;
    activate(game, source, ability);
    pass_pair(game);
    assert_eq!(
        game.state.pending,
        Pending::EffectSearch { player: Player::P0 }
    );
    source
}
fn chosen(game: &Game) -> Vec<CardId> {
    game.legal_actions()
        .into_iter()
        .filter_map(|a| {
            if let Action::ChooseSearch { card } = a {
                Some(card)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn fetch_sacrifices_before_priority_then_search_is_private_and_replayable() {
    let mut game = scenario();
    let source = enter_search(&mut game, "Evolving Wilds", 0, [0; 6]);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert!(game.state.resolving.is_some());
    assert!(game.state.stack.is_empty());
    let view = game.observe(Player::P0).search_choice.unwrap();
    assert_eq!(view.filter, crate::SearchFilter::BasicLand);
    assert_eq!(view.cards.len(), 20);
    let hidden = game.observe(Player::P1);
    assert!(hidden.search_choice.is_none());
    assert!(hidden.actions.is_empty());
    assert!(hidden.objects.iter().all(|o| o.zone != Zone::Library));
    let selected = chosen(&game)[0];
    let original = game.state.players[0].library.clone();
    let rng = game.state.rng;
    let before = game.snapshot();
    act(&mut game, Action::ChooseSearch { card: selected });
    assert_eq!(game.state.players[0].library, original);
    assert_eq!(game.state.rng, rng);
    assert_eq!(game.object_view(selected).zone, Zone::Library);
    assert!(chosen(&game).is_empty());
    assert_eq!(
        game.observe(Player::P0).search_choice.unwrap().selected,
        vec![selected]
    );
    assert!(game.observe(Player::P1).search_choice.is_none());
    let partial = game.snapshot();
    let mut branch = game.clone();
    act(&mut branch, Action::FinishSearch);
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.snapshot(), branch.snapshot());
    assert_eq!(game.object_view(selected).zone, Zone::Battlefield);
    assert!(game.object_view(selected).tapped);
    assert_eq!(game.state.players[0].library.len(), 19);
    assert_ne!(game.state.rng, rng);
    assert!(game.state.resolving.is_none());
    let after = game.snapshot();
    game.restore(&partial).unwrap();
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.snapshot(), after);
    game.draw(Player::P0);
    let future = game.snapshot();
    game.restore(&before).unwrap();
    act(&mut game, Action::ChooseSearch { card: selected });
    act(&mut game, Action::FinishSearch);
    game.draw(Player::P0);
    assert_eq!(game.snapshot(), future);
}

#[test]
fn search_rejects_wrong_zone_owner_quality_duplicate_and_excess_atomically() {
    let mut game = scenario();
    let hand = named(&mut game, Player::P0, "Forest", Zone::Hand);
    let nonbasic = named(&mut game, Player::P0, "Commercial District", Zone::Library);
    let creature = named(&mut game, Player::P0, "Grizzly Bears", Zone::Library);
    let opponent = game.state.players[1].library[0];
    enter_search(&mut game, "Burnished Hart", 0, [3, 0, 0, 0, 0, 0]);
    for bad in [hand, nonbasic, creature, opponent, u32::MAX] {
        let before = game.snapshot();
        let revision = game.revision;
        assert_eq!(
            game.apply(Player::P0, Action::ChooseSearch { card: bad }),
            Err(Error::IllegalAction)
        );
        assert_eq!(game.snapshot(), before);
        assert_eq!(game.revision, revision);
    }
    let options = chosen(&game);
    for &id in &options[..2] {
        act(&mut game, Action::ChooseSearch { card: id });
    }
    for bad in [options[0], options[2]] {
        let before = game.snapshot();
        assert_eq!(
            game.apply(Player::P0, Action::ChooseSearch { card: bad }),
            Err(Error::IllegalAction)
        );
        assert_eq!(game.snapshot(), before);
    }
}

#[test]
fn fail_to_find_is_legal_with_matches_and_still_shuffles() {
    let mut game = scenario();
    enter_search(&mut game, "Evolving Wilds", 0, [0; 6]);
    assert_eq!(chosen(&game).len(), 20);
    let count = game.state.players[0].library.len();
    let mut expected = game.state.players[0].library.clone();
    let mut rng = game.state.rng;
    rng.shuffle(&mut expected);
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.state.players[0].library, expected);
    assert_eq!(game.state.rng, rng);
    assert_eq!(game.state.players[0].library.len(), count);
    assert!(!game.state.players[0].attempted_empty_draw);
}

#[test]
fn empty_and_no_match_searches_complete_without_draw_loss_or_reveals() {
    for empty in [true, false] {
        let mut game = scenario();
        let old = game.state.players[0].library.clone();
        for id in old {
            game.move_object(id, Zone::Graveyard);
        }
        if !empty {
            named(&mut game, Player::P0, "Azorius Guildgate", Zone::Library);
        }
        enter_search(&mut game, "Evolving Wilds", 0, [0; 6]);
        assert!(chosen(&game).is_empty());
        let rng = game.state.rng;
        act(&mut game, Action::FinishSearch);
        assert!(!game.state.players[0].attempted_empty_draw);
        assert!(game.outcome().is_none());
        assert!(game.state.known_hand.is_empty());
        assert_eq!(game.state.rng, rng); // zero/one element shuffle consumes no RNG.
    }
}

#[test]
fn multi_card_search_moves_simultaneously_before_entry_triggers_and_replays() {
    let mut game = scenario();
    named(
        &mut game,
        Player::P0,
        "Eumidian Terrabotanist",
        Zone::Battlefield,
    );
    enter_search(&mut game, "Burnished Hart", 0, [3, 0, 0, 0, 0, 0]);
    let options = chosen(&game);
    act(&mut game, Action::ChooseSearch { card: options[0] });
    let partial = game.snapshot();
    act(&mut game, Action::ChooseSearch { card: options[1] });
    act(&mut game, Action::FinishSearch);
    for &id in &options[..2] {
        assert!(game.object_view(id).tapped);
    }
    assert_eq!(game.state.ordering_triggers.len(), 2);
    let after = game.snapshot();
    game.restore(&partial).unwrap();
    act(&mut game, Action::ChooseSearch { card: options[1] });
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.snapshot(), after);
}

#[test]
fn revealed_land_hand_knowledge_is_public_until_that_incarnation_leaves() {
    let mut game = scenario();
    let dual = named(&mut game, Player::P0, "Commercial District", Zone::Library);
    let wastes = named(&mut game, Player::P0, "Wastes", Zone::Library);
    let private = named(&mut game, Player::P0, "Forest", Zone::Hand);
    let world = enter_search(&mut game, "World Map", 1, [3, 0, 0, 0, 0, 0]);
    assert!(chosen(&game).contains(&dual));
    assert!(chosen(&game).contains(&wastes));
    let before = game.snapshot();
    act(&mut game, Action::ChooseSearch { card: dual });
    assert!(
        !game
            .observe(Player::P1)
            .objects
            .iter()
            .any(|o| o.id == dual)
    );
    act(&mut game, Action::FinishSearch);
    let known = game.object_ref(dual);
    for viewer in [Player::P0, Player::P1] {
        let view = game.observe(viewer);
        assert_eq!(view.known_hand, vec![known]);
        assert!(
            view.objects
                .iter()
                .any(|o| o.id == dual && o.zone == Zone::Hand)
        );
    }
    assert!(
        !game
            .observe(Player::P1)
            .objects
            .iter()
            .any(|o| o.id == private)
    );
    let after = game.snapshot();
    game.restore(&before).unwrap();
    act(&mut game, Action::ChooseSearch { card: dual });
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.snapshot(), after);
    act(&mut game, Action::PlayLand { card: dual });
    assert!(game.state.known_hand.is_empty());
    game.move_object(dual, Zone::Hand);
    assert!(
        game.observe(Player::P1)
            .objects
            .iter()
            .any(|o| o.id == dual)
    );
    assert_ne!(game.object_ref(dual), known);
    assert_eq!(game.state.known_hand, vec![game.object_ref(dual)]);
    assert_eq!(game.object_view(world).zone, Zone::Graveyard);
}

#[test]
fn lander_sacrifice_and_slot_reuse_do_not_change_pending_search_source() {
    let mut game = scenario();
    let creature = named(&mut game, Player::P0, "Galactic Wayfarer", Zone::Hand);
    game.move_object(creature, Zone::Battlefield);
    game.prepare_triggers();
    pass_pair(&mut game);
    let id = game
        .state
        .objects
        .iter()
        .position(|o| {
            o.zone == Zone::Battlefield
                && o.card
                    .token_info()
                    .is_some_and(|t| t.name == "Lander Token")
        })
        .unwrap() as CardId;
    let source = game.object_ref(id);
    game.state.players[0].mana = [2, 0, 0, 0, 0, 0];
    activate(&mut game, id, 0);
    assert_eq!(game.object_view(id).zone, Zone::Outside);
    let factory = game.state.objects[id as usize].card;
    game.create_tokens(Player::P0, factory, 1, false);
    assert_ne!(game.object_ref(id), source);
    pass_pair(&mut game);
    assert_eq!(game.state.resolving.as_ref().unwrap().item.object, source);
    let selected = chosen(&game)[0];
    act(&mut game, Action::ChooseSearch { card: selected });
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.object_view(id).zone, Zone::Battlefield);
    assert!(!game.object_view(id).tapped);
    assert!(game.object_view(selected).tapped);
}

#[test]
fn search_finishes_shuffle_before_next_instruction_and_queues_triggers_until_resolution_end() {
    let mut game = scenario();
    named(
        &mut game,
        Player::P0,
        "Eumidian Terrabotanist",
        Zone::Battlefield,
    );
    let spell = named(&mut game, Player::P0, "They Went This Way", Zone::Hand);
    game.state.players[0].mana = [2, 0, 0, 0, 1, 0];
    cast(&mut game, spell, None);
    pass_pair(&mut game);
    assert!(
        !game
            .state
            .objects
            .iter()
            .any(|o| o.zone == Zone::Battlefield && o.card.is_token())
    );
    let selected = chosen(&game)[0];
    act(&mut game, Action::ChooseSearch { card: selected });
    let mut expected = game.state.players[0].library.clone();
    expected.retain(|&id| id != selected);
    let mut rng = game.state.rng;
    rng.shuffle(&mut expected);
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.state.players[0].library, expected);
    assert_eq!(game.state.rng, rng);
    assert!(
        game.state
            .objects
            .iter()
            .any(|o| o.zone == Zone::Battlefield
                && o.card.token_info().is_some_and(|t| t.name == "Clue Token"))
    );
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(game.state.players[0].life, 20);
    assert_eq!(game.state.stack.len(), 1);
    pass_pair(&mut game);
    assert_eq!(game.state.players[0].life, 21);
}

#[test]
fn generic_search_honors_printed_entry_conditions_and_simultaneous_arrival() {
    let mut game = scenario();
    named(&mut game, Player::P0, "Forest", Zone::Battlefield);
    let slow = named(&mut game, Player::P0, "Deathcap Glade", Zone::Library);
    enter_search(&mut game, "Burnished Hart", 0, [3, 0, 0, 0, 0, 0]);
    // Private interpreter fixture for a reusable unrestricted land filter and
    // untapped-by-effect destination, independently of current card admissions.
    let spec = SearchSpec {
        filter: crate::SearchFilter::Land,
        maximum: 2,
        destination: crate::SearchDestination::Battlefield,
        tapped: false,
        reveal: false,
    };
    game.state
        .resolving
        .as_mut()
        .unwrap()
        .search
        .as_mut()
        .unwrap()
        .spec = spec;
    let basic = chosen(&game)
        .into_iter()
        .find(|&id| game.object_view(id).card.is_basic_land())
        .unwrap();
    let bytes = game.state_bytes();
    act(&mut game, Action::ChooseSearch { card: basic });
    assert_eq!(game.state_bytes(), bytes + std::mem::size_of::<CardId>());
    act(&mut game, Action::ChooseSearch { card: slow });
    act(&mut game, Action::FinishSearch);
    assert!(!game.object_view(basic).tapped);
    assert!(game.object_view(slow).tapped); // the arriving Forest wasn't present yet.
}

#[test]
fn basic_hand_search_accepts_wastes_but_rejects_typed_nonbasic_lands() {
    let mut game = scenario();
    let wastes = named(&mut game, Player::P0, "Wastes", Zone::Library);
    let typed = named(&mut game, Player::P0, "Commercial District", Zone::Library);
    enter_search(&mut game, "World Map", 0, [1, 0, 0, 0, 0, 0]);
    assert!(chosen(&game).contains(&wastes));
    assert!(!chosen(&game).contains(&typed));
    act(&mut game, Action::ChooseSearch { card: wastes });
    act(&mut game, Action::FinishSearch);
    assert!(
        game.observe(Player::P1)
            .objects
            .iter()
            .any(|o| o.id == wastes && o.zone == Zone::Hand)
    );
    assert!(
        !game
            .observe(Player::P1)
            .objects
            .iter()
            .any(|o| o.id == typed)
    );
}
