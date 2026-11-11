//! Four LCI artifact fronts and their exact crafted or transformed backs.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const TREASURE: Card = Card(5302);
const COLORLESS: Mana = [0, 0, 0, 0, 0, 1];

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

fn material(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn target(player: Player) -> Action {
    Action::ChooseTarget {
        target: Target::Player(player),
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn attack(game: &mut Game, card: CardId) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    act(game, Action::Attack { card });
    act(game, Action::FinishAttackers);
}

fn begin_craft(game: &mut Game, card: CardId, ability: u16, mana: Mana) {
    let p = game.actor().unwrap();
    game.state.players[p.index()].mana = mana;
    act(game, announce(card, ability));
    act(game, Action::BeginOrderedCosts);
    let index = game
        .observe(p)
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::Additional)
        .unwrap() as u16;
    act(game, Action::ChooseCastCost { clause: index });
}

fn finish_costs(game: &mut Game) {
    for _ in 0..8 {
        if game.state.announcing.is_none() {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::PayCosts { .. }
                    | Action::ChooseCastCost { .. }
                    | Action::FinishCraftMaterials
            )
        });
    }
    panic!("expected a completed activation");
}

fn treasures(game: &Game, controller: Player) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield
                && object.card == TREASURE
                && object.controller == controller)
                .then_some(id as CardId)
        })
        .collect()
}

#[test]
fn tithing_blade_gives_the_opponent_the_sacrifice_and_crafts_an_own_upkeep_drain() {
    let p = Player::P1;
    let enemy = p.other();
    let mut game = scenario(p);
    let card = named(&mut game, p, "Tithing Blade", Zone::Hand);
    let first = put(&mut game, enemy, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, enemy, "Grizzly Bears", Zone::Battlefield);
    let craftable = put(&mut game, p, "Bear Cub", Zone::Graveyard);
    let not_a_creature = put(&mut game, enemy, "Short Bow", Zone::Battlefield);
    let not_own = Action::Sacrifice {
        object: game.object_ref(not_a_creature),
    };
    cast(&mut game, card, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(enemy));
    reject(&mut game, not_own);
    let object = game.object_ref(second);
    act(&mut game, Action::Sacrifice { object });
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert_eq!(game.object_view(second).zone, Zone::Graveyard);
    assert_eq!(game.object_view(card).zone, Zone::Battlefield);

    let front = game.object_ref(card);
    begin_craft(&mut game, card, 0, [0, 0, 1, 0, 0, 4]);
    let invalid = material(&game, second);
    reject(&mut game, invalid);
    let choice = material(&game, craftable);
    act(&mut game, choice);
    finish_costs(&mut game);
    assert_eq!(game.object_view(card).zone, Zone::Exile);
    assert_eq!(game.object_view(craftable).zone, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.object_view(card).name, "Consuming Sepulcher");
    assert_ne!(game.object_ref(card), front);

    game.state.active = enemy;
    game.begin_step(Step::Upkeep);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].life, STARTING_LIFE);
    assert_eq!(game.state.players[enemy.index()].life, STARTING_LIFE);
    game.state.active = p;
    game.begin_step(Step::Upkeep);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].life, STARTING_LIFE + 1);
    assert_eq!(game.state.players[enemy.index()].life, STARTING_LIFE - 1);
}

fn scry(game: &mut Game, card: CardId, destination: LibraryDestination) {
    pass_pair(game);
    act(game, Action::ArrangeLibrary { card, destination });
}

#[test]
fn treasure_map_removes_all_landmark_counters_and_cove_draws_only_for_treasure() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = named(&mut game, p, "Treasure Map", Zone::Hand);
    let wrong = put(&mut game, p, "Short Bow", Zone::Battlefield);
    let draw = put(&mut game, p, "Island", Zone::Library);
    let scry_card = put(&mut game, p, "Opt", Zone::Library);
    cast(&mut game, source, None);
    settle(&mut game);
    game.add_counter(source, CounterKind::Landmark, 4);
    game.add_counter(source, CounterKind::Stun, 1);
    assert_eq!(game.object_view(source).name, "Treasure Map");
    assert!(treasures(&game, p).is_empty());
    let exact = game.object_ref(source);
    game.state.players[p.index()].mana = COLORLESS;
    act(&mut game, announce(source, 0));
    pay(&mut game);
    scry(&mut game, scry_card, LibraryDestination::Bottom);
    assert_eq!(game.object_view(source).name, "Treasure Cove");
    assert_eq!(game.object_ref(source), exact);
    assert!(game.object_view(source).tapped);
    assert_eq!(game.counter_count(source, CounterKind::Landmark), 0);
    assert_eq!(game.counter_count(source, CounterKind::Stun), 1);
    let made = treasures(&game, p);
    assert_eq!(made.len(), 3);

    game.untap_objects(&[source], untap_rules::UntapCause::Effect);
    assert!(game.object_view(source).tapped);
    game.untap_objects(&[source], untap_rules::UntapCause::Effect);
    let mut mana = game.clone();
    act(
        &mut mana,
        Action::ActivateMana {
            card: source,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: COLORLESS,
        },
    );
    assert_eq!(mana.state.players[p.index()].mana, COLORLESS);

    act(&mut game, announce(source, 1));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    let wrong = material(&game, wrong);
    reject(&mut game, wrong);
    let cost = material(&game, made[0]);
    act(&mut game, cost);
    finish_costs(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(draw).zone, Zone::Hand);
    assert!(game.object_view(source).tapped);
    assert_eq!(treasures(&game, p).len(), 2);
}

#[test]
fn treasure_map_uses_the_departed_incarnations_landmarks_after_scrying() {
    let p = Player::P1;
    for landmarks in [2, 3] {
        let mut game = scenario(p);
        let source = named(&mut game, p, "Treasure Map", Zone::Battlefield);
        let top = put(&mut game, p, "Opt", Zone::Library);
        game.add_counter(source, CounterKind::Landmark, landmarks);
        game.state.players[p.index()].mana = COLORLESS;
        act(&mut game, announce(source, 0));
        pay(&mut game);
        let exact = game.object_ref(source);
        game.move_object(source, Zone::Hand);
        game.move_object(source, Zone::Battlefield);
        assert_ne!(game.object_ref(source), exact);
        scry(&mut game, top, LibraryDestination::Top);
        assert_eq!(
            treasures(&game, p).len(),
            if landmarks == 3 { 3 } else { 0 }
        );
        assert_eq!(game.object_view(source).name, "Treasure Map");
        assert_eq!(game.counter_count(source, CounterKind::Landmark), 0);
    }
}

#[test]
fn visage_of_dread_publicly_discards_an_eligible_card_and_crafts_with_two_distinct_creatures() {
    let p = Player::P0;
    let opponent = p.other();
    let mut game = scenario(p);
    let source = named(&mut game, p, "Visage of Dread", Zone::Hand);
    let artifact = put(&mut game, opponent, "Basilisk Collar", Zone::Hand);
    let instant = put(&mut game, opponent, "Shock", Zone::Hand);
    let chosen = put(&mut game, opponent, "Grizzly Bears", Zone::Hand);
    let first_material = put(&mut game, p, "Gleaming Barrier", Zone::Battlefield);
    let second_material = put(&mut game, p, "Bear Cub", Zone::Graveyard);
    let invalid = put(&mut game, p, "Short Bow", Zone::Graveyard);
    let top_two = [
        put(&mut game, p, "Island", Zone::Library),
        put(&mut game, p, "Opt", Zone::Library),
    ];
    cast(&mut game, source, None);
    pass_pair(&mut game);
    reject(&mut game, target(p));
    act(&mut game, target(opponent));
    pass_pair(&mut game);
    for viewer in [p, opponent] {
        let observation = game.observe(viewer);
        for card in [artifact, instant, chosen] {
            assert!(observation.known_hand.contains(&game.object_ref(card)));
        }
    }
    reject(&mut game, Action::ChooseHandCard { object: None });
    let discard = |game: &Game, card| Action::ChooseHandCard {
        object: Some(game.object_ref(card)),
    };
    let wrong = discard(&game, instant);
    reject(&mut game, wrong);
    assert!(game.legal_actions().contains(&discard(&game, artifact)));
    let choice = discard(&game, chosen);
    act(&mut game, choice);
    settle(&mut game);
    assert_eq!(game.object_view(chosen).zone, Zone::Graveyard);

    begin_craft(&mut game, source, 0, [0, 0, 1, 0, 0, 5]);
    let wrong = material(&game, invalid);
    reject(&mut game, wrong);
    let first = material(&game, first_material);
    act(&mut game, first.clone());
    reject(&mut game, first);
    reject(&mut game, Action::FinishCraftMaterials);
    let second = material(&game, second_material);
    act(&mut game, second);
    finish_costs(&mut game);
    assert_eq!(game.object_view(first_material).zone, Zone::Exile);
    assert_eq!(game.object_view(second_material).zone, Zone::Exile);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.object_view(source).name, "Dread Osseosaur");
    assert_eq!(stats(&game, source), (5, 4));
    assert!(game.keywords(source).contains(Keywords::MENACE));
    act(&mut game, Action::ChooseOptional { accept: false });
    assert_eq!(game.top_library_card(p), Some(top_two[1]));

    game.state.objects[source as usize].summoning_sick = false;
    attack(&mut game, source);
    pass_pair(&mut game);
    act(&mut game, Action::ChooseOptional { accept: true });
    for card in top_two {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
}

#[test]
fn waterlogged_hulk_mills_and_crafts_with_a_nonbasic_island_into_a_descend_vehicle() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = named(&mut game, p, "Waterlogged Hulk", Zone::Hand);
    let island = put(&mut game, p, "Watery Grave", Zone::Battlefield);
    let wrong = put(&mut game, p, "Forest", Zone::Battlefield);
    let crew = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let blocker = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    for _ in 0..6 {
        put(&mut game, p, "Forest", Zone::Graveyard);
    }
    put(&mut game, p, "Shock", Zone::Graveyard);
    let top = put(&mut game, p, "Forest", Zone::Library);
    cast(&mut game, source, None);
    settle(&mut game);
    act(&mut game, announce(source, 0));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    settle(&mut game);
    assert!(game.object_view(source).tapped);
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);

    begin_craft(&mut game, source, 1, [0, 1, 0, 0, 0, 3]);
    let invalid = material(&game, wrong);
    reject(&mut game, invalid);
    let choose_island = material(&game, island);
    act(&mut game, choose_island);
    finish_costs(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(source).name, "Watertight Gondola");
    assert_eq!(game.object_view(source).kinds, 16);
    assert!(!game.object_view(source).tapped);
    act(&mut game, announce(source, 0));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    reject(&mut game, Action::FinishCrew);
    let object = game.object_ref(crew);
    act(&mut game, Action::ChooseCrewer { object });
    act(&mut game, Action::FinishCrew);
    settle(&mut game);
    assert_eq!(stats(&game, source), (4, 4));
    assert!(game.keywords(source).contains(Keywords::VIGILANCE));
    game.state.objects[source as usize].summoning_sick = false;
    let mut below = game.clone();
    attack(&mut below, source);
    pass_pair(&mut below);
    let block = Action::Block {
        blocker,
        attacker: source,
    };
    assert!(below.legal_actions().contains(&block));

    let eighth = put(&mut game, p, "Forest", Zone::Library);
    game.move_object(eighth, Zone::Graveyard);
    assert!(game.keywords(source).contains(Keywords::CANT_BE_BLOCKED));
    attack(&mut game, source);
    pass_pair(&mut game);
    assert!(!game.object_view(source).tapped);
    reject(&mut game, block);
}
