//! Transform timing, Saga chapters, and Kellan's two legal cast faces.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn named(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}

fn announce(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn tap(card: CardId, ability: u16, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn attack(game: &mut Game, attackers: &[CardId]) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

#[test]
fn dowsing_device_counts_artifacts_after_the_optional_target_and_transforms_in_place() {
    let p = Player::P0;
    let mut game = scenario(p);
    let friend = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    let first_artifact = put(&mut game, p, "Short Bow", Zone::Battlefield);
    put(&mut game, p, "Swiftfoot Boots", Zone::Battlefield);
    let device = named(&mut game, p, "Dowsing Device", Zone::Hand);
    let final_artifact = put(&mut game, p, "Basilisk Collar", Zone::Hand);
    cast(&mut game, device, None);
    pass_pair(&mut game);
    reject(&mut game, target(enemy));
    act(&mut game, target(friend));
    settle(&mut game);
    assert_eq!(game.object_view(device).name, "Dowsing Device");
    assert_eq!(stats(&game, friend), (3, 2));
    assert!(game.keywords(friend).contains(Keywords::HASTE));
    let exact = game.object_ref(device);
    cast(&mut game, final_artifact, None);
    pass_pair(&mut game);
    let mut fizzled = game.clone();
    act(&mut fizzled, target(friend));
    fizzled.move_object(friend, Zone::Hand);
    settle(&mut fizzled);
    assert_eq!(fizzled.object_view(device).name, "Dowsing Device");

    act(&mut game, Action::FinishTargets);
    settle(&mut game);
    assert_eq!(game.object_view(device).name, "Geode Grotto");
    assert_eq!(game.object_ref(device), exact);
    let red = [0, 0, 0, 1, 0, 0];
    let mut production = game.clone();
    production.state.players[p.index()].mana = [0; 6];
    act(&mut production, tap(device, 0, red));
    assert_eq!(production.state.players[p.index()].mana, red);

    game.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
    game.state.step = Step::Upkeep;
    reject(&mut game, announce(device, 1));
    game.state.step = Step::Main1;
    act(&mut game, announce(device, 1));
    act(&mut game, target(enemy));
    pay(&mut game);
    game.move_object(first_artifact, Zone::Hand);
    settle(&mut game);
    assert_eq!(stats(&game, enemy), (4, 2));
    assert!(game.keywords(enemy).contains(Keywords::HASTE));
    assert!(game.object_view(device).tapped);
}

#[test]
fn grasping_shadows_rewards_exactly_one_attacker_and_preserves_dread_through_transform() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = named(&mut game, p, "Grasping Shadows", Zone::Hand);
    let first = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    cast(&mut game, source, None);
    settle(&mut game);
    game.add_counter(source, CounterKind::Dread, 2);
    let exact = game.object_ref(source);
    let mut multiple = game.clone();
    attack(&mut multiple, &[first, second]);
    assert!(multiple.state.stack.is_empty());
    assert_eq!(multiple.counter_count(source, CounterKind::Dread), 2);

    attack(&mut game, &[first]);
    settle(&mut game);
    assert_eq!(game.object_view(source).name, "Shadows' Lair");
    assert_eq!(game.object_ref(source), exact);
    assert_eq!(game.counter_count(source, CounterKind::Dread), 3);
    assert!(
        game.keywords(first)
            .contains(Keywords::DEATHTOUCH.union(Keywords::LIFELINK))
    );
    assert!(!game.keywords(second).contains(Keywords::LIFELINK));
    let black = [0, 0, 1, 0, 0, 0];
    let mut mana = game.clone();
    mana.state.players[p.index()].mana = [0; 6];
    act(&mut mana, tap(source, 0, black));
    assert_eq!(mana.state.players[p.index()].mana, black);

    let mut counter_payment = Counters::default();
    counter_payment.add(CounterKind::Dread, 1);
    game.state.players[p.index()].mana = black;
    let draw = game.top_library_card(p).unwrap();
    act(
        &mut game,
        Action::AnnounceActivate {
            card: source,
            ability: 1,
            counter_payment,
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::Dread), 2);
    assert_eq!(game.state.players[p.index()].life, STARTING_LIFE - 1);
    assert_eq!(game.object_view(draw).zone, Zone::Hand);
    assert!(game.object_view(source).tapped);
}

#[test]
fn growing_rites_orders_the_private_remainder_and_rechecks_four_creatures_at_end_step() {
    let p = Player::P0;
    let mut game = scenario(p);
    let creatures = ["Bear Cub", "Grizzly Bears", "Savannah Lions", "Hill Giant"]
        .map(|name| put(&mut game, p, name, Zone::Battlefield));
    let source = named(&mut game, p, "Growing Rites of Itlimoc", Zone::Hand);
    let low = put(&mut game, p, "Island", Zone::Library);
    let high = put(&mut game, p, "Opt", Zone::Library);
    let middle = put(&mut game, p, "Shock", Zone::Library);
    let pick = put(&mut game, p, "Grizzly Bears", Zone::Library);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert!(game.observe(p).library_selection.is_some());
    assert!(game.observe(p.other()).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: high });
    act(&mut game, Action::SelectLibrary { card: pick });
    act(&mut game, Action::FinishLibrarySelection);
    for card in [high, middle, low] {
        act(&mut game, Action::OrderLibraryBottom { card });
    }
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    assert_eq!(game.object_view(pick).zone, Zone::Hand);
    assert_eq!(
        game.state.players[p.index()].library[..3],
        [low, middle, high]
    );
    assert!(
        game.observe(p.other())
            .known_hand
            .contains(&game.object_ref(pick))
    );

    let reference = game.object_ref(source);
    game.begin_step(Step::End);
    game.prepare_triggers();
    let mut below_threshold = game.clone();
    below_threshold.move_object(creatures[0], Zone::Hand);
    settle(&mut below_threshold);
    assert_eq!(
        below_threshold.object_view(source).name,
        "Growing Rites of Itlimoc"
    );
    settle(&mut game);
    assert_eq!(game.object_view(source).name, "Itlimoc, Cradle of the Sun");
    assert_eq!(game.object_ref(source), reference);
    game.state.players[p.index()].mana = [0; 6];
    let mut one_green = game.clone();
    act(&mut one_green, tap(source, 0, [0, 0, 0, 0, 1, 0]));
    assert_eq!(one_green.state.players[p.index()].mana[4], 1);
    act(&mut game, tap(source, 1, [0, 0, 0, 0, 4, 0]));
    assert_eq!(game.state.players[p.index()].mana[4], 4);
}

fn search_for(game: &mut Game, card: CardId) {
    for _ in 0..8 {
        if game
            .legal_actions()
            .contains(&Action::ChooseSearch { card })
        {
            act(game, Action::ChooseSearch { card });
            if game.legal_actions().contains(&Action::FinishSearch) {
                act(game, Action::FinishSearch);
            }
            return;
        }
        act(game, Action::Pass);
    }
    panic!("expected a search for card {card}");
}

#[test]
fn huatli_pays_hybrid_to_return_as_a_saga_and_resolves_all_four_chapters() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = named(&mut game, p, "Huatli, Poet of Unity", Zone::Hand);
    let basic = put(&mut game, p, "Island", Zone::Library);
    let dinosaur = put(&mut game, p, "Hulking Raptor", Zone::Library);
    let recipient = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    cast(&mut game, source, None);
    search_for(&mut game, basic);
    assert_eq!(game.object_view(basic).zone, Zone::Hand);
    let original = game.object_ref(source);
    game.state.players[p.index()].mana = [1, 0, 0, 1, 0, 3];
    act(&mut game, announce(source, 0));
    if game
        .legal_actions()
        .iter()
        .any(|action| matches!(action, Action::ChooseActivationCost { .. }))
    {
        choose(&mut game, |action| {
            matches!(action, Action::ChooseActivationCost { .. })
        });
    }
    pay(&mut game);
    settle(&mut game);
    let saga = game.object_ref(source);
    assert_ne!(saga, original);
    assert_eq!(game.object_view(source).name, "Roar of the Fifth People");
    assert_eq!(game.counter_count(source, CounterKind::Lore), 1);
    let tokens: Vec<_> = game
        .state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.card == Card(5327) && object.zone == Zone::Battlefield).then_some(id as CardId)
        })
        .collect();
    assert_eq!(tokens.len(), 2);
    for &token in &tokens {
        assert_eq!(stats(&game, token), (3, 3));
        assert!(game.has_subtype(token, "Dinosaur"));
    }
    let ability = |card, production| Action::TapGrantedMana {
        card,
        grantor: saga,
        production,
    };
    let white = [1, 0, 0, 0, 0, 0];
    reject(&mut game, ability(recipient, white));
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.counter_count(source, CounterKind::Lore), 2);
    reject(&mut game, ability(enemy, white));
    reject(&mut game, ability(recipient, [0, 1, 0, 0, 0, 0]));
    act(&mut game, ability(recipient, white));
    assert_eq!(game.state.players[p.index()].mana, white);

    game.begin_step(Step::Main1);
    game.prepare_triggers();
    search_for(&mut game, dinosaur);
    assert_eq!(game.object_view(dinosaur).zone, Zone::Hand);
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    let keywords = Keywords::DOUBLE_STRIKE.union(Keywords::TRAMPLE);
    for token in tokens {
        assert!(game.keywords(token).contains(keywords));
    }
    assert!(!game.keywords(recipient).contains(Keywords::DOUBLE_STRIKE));
}

#[test]
fn journey_on_creates_one_map_plus_one_for_an_artifact_controlling_opponent() {
    let p = Player::P0;
    for opponents_artifacts in [0, 2] {
        let mut game = scenario(p);
        let spell = named(&mut game, p, "Kellan, Daring Traveler", Zone::Hand);
        put(&mut game, p, "Short Bow", Zone::Battlefield);
        for _ in 0..opponents_artifacts {
            put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
        }
        game.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        act(&mut game, Action::AnnounceAdventure { card: spell });
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(spell).zone, Zone::Exile);
        let maps = game
            .state
            .objects
            .iter()
            .filter(|object| object.card == Card(5284) && object.zone == Zone::Battlefield)
            .count();
        assert_eq!(maps, if opponents_artifacts == 0 { 1 } else { 2 });
        game.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
        act(&mut game, Action::AnnounceExile { card: spell });
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(spell).zone, Zone::Battlefield);
        assert_eq!(stats(&game, spell), (2, 3));
    }
}

#[test]
fn kellan_takes_an_eligible_revealed_creature_and_offers_to_mill_a_larger_one() {
    let p = Player::P1;
    for (name, eligible) in [("Bear Cub", true), ("Hill Giant", false)] {
        let mut game = scenario(p);
        let source = named(&mut game, p, "Kellan, Daring Traveler", Zone::Battlefield);
        let top = put(&mut game, p, name, Zone::Library);
        attack(&mut game, &[source]);
        pass_pair(&mut game);
        if eligible {
            if game
                .legal_actions()
                .contains(&Action::SelectLibrary { card: top })
            {
                assert!(
                    !game
                        .legal_actions()
                        .contains(&Action::FinishLibrarySelection)
                );
                act(&mut game, Action::SelectLibrary { card: top });
                act(&mut game, Action::FinishLibrarySelection);
            }
            assert_eq!(game.object_view(top).zone, Zone::Hand);
            assert!(
                game.observe(p.other())
                    .known_hand
                    .contains(&game.object_ref(top))
            );
        } else {
            assert!(game.observe(p.other()).library_selection.unwrap().public);
            act(&mut game, Action::FinishLibrarySelection);
            let mut kept = game.clone();
            act(&mut kept, Action::FinishLibrarySelection);
            assert_eq!(kept.top_library_card(p), Some(top));
            act(&mut game, Action::OrderLibraryBottom { card: top });
            if game
                .legal_actions()
                .contains(&Action::FinishLibrarySelection)
            {
                act(&mut game, Action::FinishLibrarySelection);
            }
            assert_eq!(game.object_view(top).zone, Zone::Graveyard);
        }
    }
}
