use super::support::*;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};

const CLUE: Card = Card::from_catalogue_id(5296);
const THOPTER: Card = Card::from_catalogue_id(5266);

fn begin_split(game: &mut Game, source: CardId, half: SplitHalf) {
    act(
        game,
        Action::AnnounceSplit {
            card: source,
            half,
            method: CastMethod::Normal,
            flash_surcharge: false,
        },
    );
    choose(game, |action| {
        matches!(action, Action::ChooseSpellCost { .. })
    });
}

fn choose_card(game: &mut Game, card: CardId) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
}

fn pay_exact(game: &mut Game, payment: Mana) {
    act(game, Action::PayCosts { payment });
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn token_count(game: &Game, player: Player, recipe: Card) -> usize {
    game.controlled_token_ids(player)
        .filter(|&id| game.object_view(id).card == recipe)
        .count()
}

#[test]
fn cease_and_desist_have_separate_costs_timing_and_effects() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let first = put(&mut game, player, "Shock", Zone::Graveyard);
    let second = put(&mut game, player, "Forest", Zone::Graveyard);
    let foreign = put(&mut game, opponent, "Mountain", Zone::Graveyard);
    let cease = put(&mut game, player, "Cease // Desist", Zone::Hand);
    assert_eq!(game.object_view(cease).card.fixed_mana_value(), 8);
    game.state.active = opponent;
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 1];
    begin_split(&mut game, cease, SplitHalf::Left);
    choose_card(&mut game, first);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(foreign),
        },
    );
    choose_card(&mut game, second);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    pay_exact(&mut game, [0, 0, 0, 0, 1, 1]);
    assert_eq!(
        game.state.stack.last().unwrap().cast.unwrap().face,
        CastFace::SplitLeft
    );
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Exile);
    assert_eq!(game.object_view(second).zone, Zone::Exile);
    assert_eq!(game.object_view(foreign).zone, Zone::Graveyard);
    assert_eq!(game.state.players[opponent.index()].life, 22);
    assert_eq!(
        game.state.players[opponent.index()].cards_drawn_this_turn,
        1
    );

    game.state.active = player;
    game.state.priority = player;
    let artifact = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let enchantment = put(&mut game, opponent, "Doubling Season", Zone::Battlefield);
    let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let desist = put(&mut game, player, "Cease // Desist", Zone::Hand);
    game.state.players[player.index()].mana = [1, 0, 0, 0, 1, 4];
    begin_split(&mut game, desist, SplitHalf::Right);
    pay_exact(&mut game, [1, 0, 0, 0, 1, 4]);
    settle(&mut game);
    for id in [artifact, enchantment] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
    assert!(game.on_battlefield(creature));
}

#[test]
fn flotsam_investigates_after_own_mill_and_jetsam_casts_from_opponent_graveyard() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let flotsam = put(&mut game, player, "Flotsam // Jetsam", Zone::Hand);
    let before = game.state.players[player.index()].library.len();
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 1];
    begin_split(&mut game, flotsam, SplitHalf::Left);
    pay_exact(&mut game, [0, 1, 0, 0, 0, 1]);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].library.len(), before - 3);
    assert_eq!(token_count(&game, player, CLUE), 1);
    let own_shock = put(&mut game, player, "Shock", Zone::Graveyard);
    let shock = put(&mut game, opponent, "Shock", Zone::Library);
    let before = game.state.players[opponent.index()].library.len();
    let jetsam = put(&mut game, player, "Flotsam // Jetsam", Zone::Hand);
    game.state.players[player.index()].mana = [0, 1, 1, 0, 0, 4];
    begin_split(&mut game, jetsam, SplitHalf::Right);
    pay_exact(&mut game, [0, 1, 1, 0, 0, 4]);
    pass_pair(&mut game);
    assert_eq!(
        game.state.players[opponent.index()].library.len(),
        before - 3
    );
    let offer = |g: &Game, id| Action::CastExiledDuringResolution {
        object: g.object_ref(id),
        face: CastFace::Normal,
    };
    let wrong = offer(&game, own_shock);
    reject(&mut game, wrong);
    let mut declined = game.clone();
    act(&mut declined, Action::FinishResolutionCasts);
    assert_eq!(declined.object_view(shock).zone, Zone::Graveyard);
    let action = offer(&game, shock);
    act(&mut game, action);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    if game
        .legal_actions()
        .iter()
        .any(|a| matches!(a, Action::PayCosts { .. }))
    {
        pay(&mut game);
    }
    if game
        .legal_actions()
        .contains(&Action::FinishResolutionCasts)
    {
        act(&mut game, Action::FinishResolutionCasts);
    }
    let receipt = game.state.stack.last().unwrap().cast.unwrap();
    assert_eq!(receipt.method, CastMethod::Free);
    assert_eq!(receipt.origin, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(shock).zone, Zone::Exile);
    assert_eq!(game.state.players[opponent.index()].life, 18);
}

#[test]
fn fuss_counts_attackers_and_bother_makes_flying_thopters_before_surveil() {
    let player = Player::P0;
    let mut combat = scenario(player);
    let attacker = put(&mut combat, player, "Grizzly Bears", Zone::Battlefield);
    let idle = put(&mut combat, player, "Bear Cub", Zone::Battlefield);
    combat.state.step = Step::Attackers;
    combat.state.pending = Pending::Attackers;
    act(&mut combat, Action::Attack { card: attacker });
    act(&mut combat, Action::FinishAttackers);
    let fuss = put(&mut combat, player, "Fuss // Bother", Zone::Hand);
    combat.state.players[player.index()].mana = [1, 0, 0, 0, 0, 2];
    begin_split(&mut combat, fuss, SplitHalf::Left);
    pay_exact(&mut combat, [1, 0, 0, 0, 0, 2]);
    settle(&mut combat);
    assert_eq!(combat.counter_count(attacker, CounterKind::PlusOne), 1);
    assert_eq!(combat.counter_count(idle, CounterKind::PlusOne), 0);

    let mut game = scenario(player);
    let keep = put(&mut game, player, "Forest", Zone::Library);
    let mill = put(&mut game, player, "Shock", Zone::Library);
    let bother = put(&mut game, player, "Fuss // Bother", Zone::Hand);
    game.state.players[player.index()].mana = [1, 1, 0, 0, 0, 4];
    begin_split(&mut game, bother, SplitHalf::Right);
    pay_exact(&mut game, [1, 1, 0, 0, 0, 4]);
    pass_pair(&mut game);
    assert_eq!(token_count(&game, player, THOPTER), 3);
    for id in game.controlled_token_ids(player) {
        let thopter = game.object_view(id);
        assert_eq!(thopter.kinds, 1 | 16);
        assert_eq!((thopter.power, thopter.toughness), (1, 1));
        assert!(thopter.keywords.contains(Keywords::FLYING));
    }
    for (card, destination) in [
        (mill, LibraryDestination::Graveyard),
        (keep, LibraryDestination::Top),
    ] {
        act(&mut game, Action::ArrangeLibrary { card, destination });
    }
    assert_eq!(game.object_view(mill).zone, Zone::Graveyard);
    assert_eq!(game.top_library_card(player), Some(keep));
}

#[test]
fn hustle_requires_a_legal_attack_and_bustle_can_turn_up_a_buffed_creature() {
    let player = Player::P0;
    let mut game = scenario(player);
    let attacker = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let hustle = put(&mut game, player, "Hustle // Bustle", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
    begin_split(&mut game, hustle, SplitHalf::Left);
    choose_card(&mut game, attacker);
    pay_exact(&mut game, [0, 0, 0, 1, 0, 0]);
    settle(&mut game);
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    reject(&mut game, Action::FinishAttackers);
    act(&mut game, Action::Attack { card: attacker });
    act(&mut game, Action::FinishAttackers);

    let mut game = scenario(player);
    let hidden = put(&mut game, player, "Grizzly Bears", Zone::Library);
    game.cloak_library_top(player, player);
    assert!(!game.advance_entries());
    let object = game.object_ref(hidden);
    let opponent = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let bustle = put(&mut game, player, "Hustle // Bustle", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 1, 1, 4];
    begin_split(&mut game, bustle, SplitHalf::Right);
    pay_exact(&mut game, [0, 0, 0, 1, 1, 4]);
    pass_pair(&mut game);
    assert_eq!(game.stats(hidden).power, 4);
    assert!(game.keywords(hidden).contains(Keywords::TRAMPLE));
    assert_eq!(game.stats(opponent).power, 2);
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineEffectPayment);
    assert!(decline.is_face_down(object));
    act(&mut game, Action::ChooseEffectObject { object });
    assert!(!game.is_face_down(object));
    assert_eq!(game.object_ref(hidden), object);
    assert_eq!(game.stats(hidden).power, 4);
    assert!(game.keywords(hidden).contains(Keywords::TRAMPLE));
}

#[test]
fn push_rechecks_tapped_and_pull_returns_exact_creatures_with_haste_and_a_delay() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let tapped = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[tapped as usize].tapped = true;
    let untapped = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let push = put(&mut game, player, "Push // Pull", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 1, 0, 0, 1];
    begin_split(&mut game, push, SplitHalf::Left);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(untapped),
        },
    );
    choose_card(&mut game, tapped);
    pay_exact(&mut game, [0, 0, 1, 0, 0, 1]);
    let mut fails = game.clone();
    fails.state.objects[tapped as usize].tapped = false;
    settle(&mut fails);
    assert!(fails.on_battlefield(tapped));
    settle(&mut game);
    assert_eq!(game.object_view(tapped).zone, Zone::Graveyard);

    let second = put(&mut game, opponent, "Bear Cub", Zone::Graveyard);
    let own = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let pull = put(&mut game, player, "Push // Pull", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 1, 1, 0, 4];
    begin_split(&mut game, pull, SplitHalf::Right);
    choose_card(&mut game, tapped);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(own),
        },
    );
    choose_card(&mut game, second);
    pay_exact(&mut game, [0, 0, 1, 1, 0, 4]);
    settle(&mut game);
    for id in [tapped, second] {
        let object = game.object_view(id);
        assert_eq!(object.zone, Zone::Battlefield);
        assert_eq!(object.controller, player);
        assert_eq!(object.owner, opponent);
        assert!(object.keywords.contains(Keywords::HASTE));
    }
    game.move_object(tapped, Zone::Exile);
    game.move_object(tapped, Zone::Battlefield);
    game.state.step = Step::Main2;
    pass_pair(&mut game);
    settle(&mut game);
    assert!(game.on_battlefield(tapped));
    assert_eq!(game.object_view(second).zone, Zone::Graveyard);
}
