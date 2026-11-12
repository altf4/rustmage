use super::*;

const fn cost(colored: Mana, generic: u32) -> ManaCost {
    ManaCost { colored, generic }
}

const GREEN: Mana = [0, 0, 0, 0, 1, 0];

fn disguise(game: &mut Game, player: Player, name: &str) -> CardId {
    let card = put(game, player, name, Zone::Hand);
    act(game, Action::AnnounceDisguise { card });
    pay(game);
    settle(game);
    assert_eq!(stats(game, card), (2, 2));
    card
}

fn turn_up(game: &mut Game, card: CardId, cost: ManaCost) {
    let object = game.object_ref(card);
    act(game, Action::BeginFaceUp { object, cost });
    choose(game, |action| matches!(action, Action::PayFaceUp { .. }));
    assert_eq!(game.object_ref(card), object);
}

fn must_block(game: &mut Game, attacker: CardId, blocker: CardId) {
    attack(game, &[attacker]);
    pass_pair(game);
    reject(game, Action::FinishBlockers);
    act(game, Action::Block { blocker, attacker });
    act(game, Action::FinishBlockers);
}

#[test]
fn culvert_ambusher_requires_a_block_from_either_entry_event() {
    let player = Player::P0;
    let mut root = scenario(player);
    let attacker = put(&mut root, player, "Bear Cub", Zone::Battlefield);
    let blocker = put(&mut root, player.other(), "Hill Giant", Zone::Battlefield);
    let mut face_up = root.clone();
    let ambusher = put(&mut face_up, player, "Culvert Ambusher", Zone::Hand);
    cast(&mut face_up, ambusher, None);
    pass_pair(&mut face_up);
    act(&mut face_up, target(blocker));
    settle(&mut face_up);
    must_block(&mut face_up, attacker, blocker);

    let hidden = disguise(&mut root, player, "Culvert Ambusher");
    let mut no_requirement = root.clone();
    attack(&mut no_requirement, &[attacker]);
    pass_pair(&mut no_requirement);
    act(&mut no_requirement, Action::FinishBlockers);
    turn_up(&mut root, hidden, cost(GREEN, 4));
    act(&mut root, target(blocker));
    settle(&mut root);
    must_block(&mut root, attacker, blocker);
}

#[test]
fn flourishing_bloom_kin_counts_nonbasic_forests_and_splits_two_found_cards() {
    let player = Player::P1;
    let mut game = scenario(player);
    put(&mut game, player, "Forest", Zone::Battlefield);
    put(&mut game, player, "Breeding Pool", Zone::Battlefield);
    put(&mut game, player.other(), "Forest", Zone::Battlefield);
    let enter = put(&mut game, player, "Forest", Zone::Library);
    let hand = put(&mut game, player, "Temple Garden", Zone::Library);
    let other = put(&mut game, player, "Island", Zone::Library);
    let source = disguise(&mut game, player, "Flourishing Bloom-Kin");
    turn_up(&mut game, source, cost(GREEN, 4));
    assert_eq!(stats(&game, source), (2, 2));
    pass_pair(&mut game);
    reject(&mut game, Action::ChooseSearch { card: other });
    let mut none = game.clone();
    act(&mut none, Action::FinishSearch);
    assert_eq!(stats(&none, source), (2, 2));
    for card in [enter, hand] {
        act(&mut game, Action::ChooseSearch { card });
    }
    act(&mut game, Action::FinishSearch);
    act(&mut game, Action::ChooseSearchBattlefield { card: enter });
    assert_eq!(game.object_view(enter).zone, Zone::Battlefield);
    assert!(game.object_view(enter).tapped);
    assert_eq!(game.object_view(hand).zone, Zone::Hand);
    assert!(
        game.observe(player.other())
            .known_hand
            .iter()
            .any(|object| object.card == hand)
    );
    assert_eq!(stats(&game, source), (3, 3));
}

#[test]
fn greenbelt_radical_counters_the_current_team_and_grants_temporary_trample() {
    let player = Player::P0;
    let mut game = scenario(player);
    let controlled = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let foreign = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let source = disguise(&mut game, player, "Greenbelt Radical");
    turn_up(&mut game, source, cost([0, 0, 0, 0, 2, 0], 5));
    settle(&mut game);
    for card in [source, controlled] {
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 1);
        assert!(game.keywords(card).contains(Keywords::TRAMPLE));
    }
    assert_eq!(game.counter_count(foreign, CounterKind::PlusOne), 0);
    let late = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    assert!(!game.keywords(late).contains(Keywords::TRAMPLE));
    game.finish_cleanup();
    assert!(!game.keywords(source).contains(Keywords::TRAMPLE));
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
}

#[test]
fn hedge_whisperer_uses_evidence_and_ends_its_exact_form_on_first_untap() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Hedge Whisperer", Zone::Battlefield);
    let mountain = put(&mut game, player, "Mountain", Zone::Battlefield);
    let other = put(&mut game, player.other(), "Forest", Zone::Battlefield);
    let fuel = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let mut off_turn = game.clone();
    off_turn.state.active = player.other();
    reject(&mut off_turn, activation(source, 0));
    act(&mut game, activation(source, 0));
    reject(&mut game, target(other));
    act(&mut game, target(mountain));
    let object = game.object_ref(fuel);
    act(&mut game, Action::ChooseEvidence { object });
    act(&mut game, Action::FinishEvidence);
    pay(&mut game);
    assert!(game.object_view(source).tapped);
    assert_eq!(game.object_view(fuel).zone, Zone::Exile);

    let mut source_gone = game.clone();
    source_gone.move_object(source, Zone::Hand);
    settle(&mut source_gone);
    assert!(!source_gone.is_creature(mountain));

    settle(&mut game);
    let land = game.object_view(mountain);
    assert_eq!(
        (land.kinds, land.colors, land.power, land.toughness),
        (3, 16, 5, 5)
    );
    assert!(land.keywords.contains(Keywords::HASTE));
    assert!(
        ["Mountain", "Plant", "Boar"]
            .iter()
            .all(|s| land.has_subtype(s))
    );
    let red = game.state.players[player.index()].mana[3];
    act(&mut game, Action::TapMana { card: mountain });
    assert_eq!(game.state.players[player.index()].mana[3], red + 1);
    for _ in 0..40 {
        if game.state.step == Step::Untap {
            break;
        }
        choose(&mut game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishAttackers | Action::FinishBlockers
            )
        });
    }
    assert_eq!(
        game.observe(player).untap_choice,
        Some(game.object_ref(source))
    );
    let mut keep = game.clone();
    act(&mut keep, Action::ChooseOptional { accept: false });
    assert!(keep.is_creature(mountain));
    act(&mut game, Action::ChooseOptional { accept: true });
    assert!(!game.is_creature(mountain));
}

#[test]
fn hide_in_plain_sight_privately_cloaks_two_and_bottoms_the_rest() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let [outside, first, second, third, fourth, fifth] = [
            "Bear Cub",
            "Hill Giant",
            "Shock",
            "Island",
            "Grizzly Bears",
            "Opt",
        ]
        .map(|name| put(&mut game, player, name, Zone::Library));
        let spell = put(&mut game, player, "Hide in Plain Sight", Zone::Hand);
        cast(&mut game, spell, None);
        pass_pair(&mut game);
        assert_eq!(
            game.observe(player)
                .library_selection
                .unwrap()
                .remaining
                .len(),
            5
        );
        assert!(game.observe(player.other()).library_selection.is_none());
        reject(&mut game, Action::SelectLibrary { card: outside });
        act(&mut game, Action::SelectLibrary { card: first });
        reject(&mut game, Action::FinishLibrarySelection);
        act(&mut game, Action::SelectLibrary { card: second });
        act(&mut game, Action::FinishLibrarySelection);
        for card in [first, second] {
            assert!(game.on_battlefield(card));
            assert_eq!(stats(&game, card), (2, 2));
        }
        assert!(
            game.observe(player.other())
                .face_down_permanents
                .iter()
                .all(|object| object.underlying.is_none())
        );
        let library = &game.state.players[player.index()].library;
        assert_eq!(library.last(), Some(&outside));
        let mut bottom = library[..3].to_vec();
        bottom.sort_unstable();
        assert_eq!(bottom, [third, fourth, fifth]);
    }
}

#[test]
fn nervous_gardener_searches_any_basic_land_type_including_a_nonbasic() {
    let player = Player::P0;
    let mut game = scenario(player);
    let typed = put(&mut game, player, "Breeding Pool", Zone::Library);
    let untyped = put(&mut game, player, "Cavern of Souls", Zone::Library);
    let source = disguise(&mut game, player, "Nervous Gardener");
    turn_up(&mut game, source, cost(GREEN, 0));
    pass_pair(&mut game);
    reject(&mut game, Action::ChooseSearch { card: untyped });
    act(&mut game, Action::ChooseSearch { card: typed });
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.object_view(typed).zone, Zone::Hand);
    assert!(
        game.observe(player.other())
            .known_hand
            .iter()
            .any(|object| object.card == typed)
    );
}

#[test]
fn tunnel_tipster_remembers_a_prior_face_down_entry_and_adds_green_mana() {
    let player = Player::P1;
    let mut game = scenario(player);
    let hidden = disguise(&mut game, player, "Nervous Gardener");
    let source = put(&mut game, player, "Tunnel Tipster", Zone::Battlefield);
    let before = game.state.players[player.index()].mana[4];
    choose(
        &mut game,
        |action| matches!(action, Action::ActivateMana { card, production: [0, 0, 0, 0, 1, 0], .. } if *card == source),
    );
    assert_eq!(game.state.players[player.index()].mana[4], before + 1);
    turn_up(&mut game, hidden, cost(GREEN, 0));
    pass_pair(&mut game);
    act(&mut game, Action::FinishSearch);
    game.state.step = Step::Main2;
    pass_pair(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);

    let mut quiet = scenario(player);
    let source = put(&mut quiet, player, "Tunnel Tipster", Zone::Battlefield);
    quiet.state.step = Step::Main2;
    pass_pair(&mut quiet);
    settle(&mut quiet);
    assert_eq!(quiet.counter_count(source, CounterKind::PlusOne), 0);
}

#[test]
fn vengeful_creeper_only_destroys_a_current_opposing_artifact_or_enchantment() {
    let player = Player::P0;
    let mut game = scenario(player);
    let own = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let foe = put(
        &mut game,
        player.other(),
        "Impact Tremors",
        Zone::Battlefield,
    );
    let source = disguise(&mut game, player, "Vengeful Creeper");
    turn_up(&mut game, source, cost(GREEN, 5));
    reject(&mut game, target(own));
    act(&mut game, target(foe));
    let mut protected = game.clone();
    protected.state.objects[foe as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    settle(&mut protected);
    assert!(protected.on_battlefield(foe));
    settle(&mut game);
    assert_eq!(game.object_view(foe).zone, Zone::Graveyard);
}

#[test]
fn crowd_control_warden_replaces_entry_and_turn_up_using_the_current_other_creatures() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    put(&mut game, player, "Bear Cub", Zone::Battlefield);
    put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);

    let mut ordinary = game.clone();
    let source = put(&mut ordinary, player, "Crowd-Control Warden", Zone::Hand);
    cast(&mut ordinary, source, None);
    settle(&mut ordinary);
    assert_eq!(ordinary.counter_count(source, CounterKind::PlusOne), 2);

    let source = disguise(&mut game, player, "Crowd-Control Warden");
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 0);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.state.players[player.index()].mana = [1, 0, 0, 0, 1, 3];
    turn_up(&mut game, source, cost([1, 0, 0, 0, 1, 0], 3));
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 4);
    assert_eq!(stats(&game, source), (8, 8));
}

#[test]
fn dog_walker_and_gadget_technician_witness_their_own_token_events() {
    for (name, hybrid, factory, entry, turned, keywords) in [
        ("Dog Walker", [1, 0, 0, 1, 0, 0], 5256, 0, 2, Keywords::NONE),
        (
            "Gadget Technician",
            [0, 1, 0, 1, 0, 0],
            5266,
            1,
            1,
            Keywords::FLYING,
        ),
    ] {
        let player = Player::P1;
        let mut game = scenario(player);
        let mut face_up = game.clone();
        let card = put(&mut face_up, player, name, Zone::Hand);
        cast(&mut face_up, card, None);
        settle(&mut face_up);
        assert_eq!(tokens(&face_up, player, factory).len(), entry);

        let source = disguise(&mut game, player, name);
        assert!(tokens(&game, player, factory).is_empty());
        game.state.players[player.index()].mana = hybrid;
        turn_up(&mut game, source, cost(hybrid, 0));
        settle(&mut game);
        let made = tokens(&game, player, factory);
        assert_eq!(made.len(), turned);
        for token in made {
            let view = game.object_view(token);
            assert_eq!((view.power, view.toughness), (1, 1));
            assert_eq!(view.tapped, name == "Dog Walker");
            assert!(view.keywords.contains(keywords));
            assert_eq!(view.kinds, if name == "Dog Walker" { 1 } else { 1 | 16 });
        }
        if name == "Dog Walker" {
            assert!(game.keywords(source).contains(Keywords::VIGILANCE));
        }
    }
}

#[test]
fn etrata_cloaks_the_damaged_players_card_then_turns_it_up_or_casts_it_free() {
    for (name, creature) in [("Bear Cub", true), ("Shock", false)] {
        let player = Player::P1;
        let opponent = player.other();
        let mut game = scenario(player);
        let source = put(
            &mut game,
            player,
            "Etrata, Deadly Fugitive",
            Zone::Battlefield,
        );
        let hidden = put(&mut game, opponent, name, Zone::Library);
        assert!(game.keywords(source).contains(Keywords::DEATHTOUCH));
        attack(&mut game, &[source]);
        pass_pair(&mut game);
        act(&mut game, Action::FinishBlockers);
        pass_pair(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(hidden).controller, player);
        assert_eq!(
            game.observe(opponent).face_down_permanents[0].underlying,
            None
        );
        assert!(
            game.observe(player).face_down_permanents[0]
                .underlying
                .is_some()
        );

        let object = game.object_ref(hidden);
        let grantor = game.object_ref(source);
        game.state.players[player.index()].mana = [0, 1, 1, 0, 0, 2];
        reject(&mut game, activation(source, 0));
        act(
            &mut game,
            Action::AnnounceCreatureGrant {
                source: object,
                grantor,
                ability: 0,
            },
        );
        pay(&mut game);
        pass_pair(&mut game);
        if creature {
            assert_eq!(game.object_ref(hidden), object);
            assert!(game.observe(player).face_down_permanents.is_empty());
            assert_eq!(game.object_view(hidden).controller, player);
        } else {
            assert_eq!(game.object_view(hidden).zone, Zone::Exile);
            let object = game.object_ref(hidden);
            act(
                &mut game,
                Action::CastExiledDuringResolution {
                    object,
                    face: CastFace::Normal,
                },
            );
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Player(opponent),
                },
            );
            if matches!(game.state.pending, Pending::PayCosts { .. }) {
                pay(&mut game);
            }
            act(&mut game, Action::FinishResolutionCasts);
            settle(&mut game);
            assert_eq!(game.state.players[opponent.index()].life, 17);
            assert_eq!(game.object_view(hidden).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn faerie_snoop_privately_puts_one_card_in_hand_and_the_other_in_graveyard() {
    let player = Player::P1;
    let mut game = scenario(player);
    let hand = put(&mut game, player, "Bear Cub", Zone::Library);
    let grave = put(&mut game, player, "Hill Giant", Zone::Library);
    let source = disguise(&mut game, player, "Faerie Snoop");
    turn_up(&mut game, source, cost([0, 1, 1, 0, 0, 0], 1));
    pass_pair(&mut game);
    assert!(game.keywords(source).contains(Keywords::FLYING));
    assert!(game.observe(player.other()).library_selection.is_none());
    act(&mut game, Action::SelectLibrary { card: hand });
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(game.object_view(hand).zone, Zone::Hand);
    assert_eq!(game.object_view(grave).zone, Zone::Graveyard);
    assert!(
        game.observe(player.other())
            .known_hand
            .iter()
            .all(|object| object.card != hand)
    );
}

#[test]
fn granite_witness_can_leave_the_target_or_change_its_tapped_state() {
    for tapped in [false, true] {
        let player = Player::P0;
        let mut game = scenario(player);
        let recipient = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
        game.state.objects[recipient as usize].tapped = tapped;
        let source = disguise(&mut game, player, "Granite Witness");
        turn_up(&mut game, source, cost([1, 1, 0, 0, 0, 0], 0));
        assert!(
            game.keywords(source)
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );
        act(&mut game, target(recipient));
        pass_pair(&mut game);
        let mut decline = game.clone();
        act(&mut decline, Action::ChooseOptional { accept: false });
        assert_eq!(decline.object_view(recipient).tapped, tapped);
        act(&mut game, Action::ChooseOptional { accept: true });
        assert_eq!(game.object_view(recipient).tapped, !tapped);
    }
}
