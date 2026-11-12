//! Multi-target resolution, combat tricks, and typed modal outcomes.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn spell(game: &mut Game, player: Player, name: &str, targets: &[CardId]) -> CardId {
    game.state.players[player.index()].mana = [8; 6];
    let card = put(game, player, name, Zone::Hand);
    if targets.len() <= 1 {
        cast(game, card, targets.first().copied().map(Target::Card));
    } else {
        let targets: Vec<_> = targets.iter().copied().map(Target::Card).collect();
        announce(game, card, &targets);
    }
    card
}

fn mode(game: &mut Game, player: Player, name: &str, mode: u16, chosen: Option<CardId>) {
    game.state.players[player.index()].mana = [8; 6];
    let card = put(game, player, name, Zone::Hand);
    act(game, Action::AnnounceCast { card });
    act(game, Action::ChooseMode { mode });
    if let Some(chosen) = chosen {
        act(game, target(chosen));
    }
    pay(game);
    settle(game);
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn conduct_electricity_combines_two_occurrences_on_one_token() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    game.create_tokens(opponent, Card::from_catalogue_id(5324), 1, false);
    assert!(!game.advance_entries());
    let token = game.controlled_token_ids(opponent).next().unwrap();
    game.add_counter(token, CounterKind::PlusOne, 6);
    let bear = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let conduct = put(&mut game, player, "Conduct Electricity", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: conduct });
    act(&mut game, target(token));
    reject(&mut game, target(bear));
    let mut one = game.clone();
    act(&mut one, Action::FinishTargets);
    pay(&mut one);
    settle(&mut one);
    assert_eq!(one.object_view(token).damage, 6);
    act(&mut game, target(token));
    pay(&mut game);
    settle(&mut game);
    assert!(!game.on_battlefield(token));
}

#[test]
fn recruiter_needs_small_power_or_another_lizard_and_meek_needs_a_mouse() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let big = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    game.state.objects[big as usize].tapped = true;
    let mut no_lizard = game.clone();
    put(&mut game, player, "Flamecache Gecko", Zone::Battlefield);
    for branch in [&mut game, &mut no_lizard] {
        let recruiter = spell(branch, player, "Reptilian Recruiter", &[]);
        pass_pair(branch);
        act(branch, target(big));
        settle(branch);
        assert!(branch.keywords(recruiter).contains(Keywords::TRAMPLE));
    }
    assert_eq!(no_lizard.object_view(big).controller, opponent);
    assert!(no_lizard.object_view(big).tapped);
    assert_eq!(game.object_view(big).controller, player);
    assert!(!game.object_view(big).tapped);
    assert!(game.keywords(big).contains(Keywords::HASTE));
    let mut no_mouse = game.clone();
    spell(&mut no_mouse, player, "Might of the Meek", &[big]);
    settle(&mut no_mouse);
    assert_eq!(stats(&no_mouse, big), (3, 3));
    assert!(no_mouse.keywords(big).contains(Keywords::TRAMPLE));
    put(&mut game, player, "Whiskerquill Scribe", Zone::Battlefield);
    spell(&mut game, player, "Might of the Meek", &[big]);
    settle(&mut game);
    assert_eq!(stats(&game, big), (4, 3));
    assert!(game.keywords(big).contains(Keywords::TRAMPLE));
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    game.finish_cleanup();
    assert_eq!(game.object_view(big).controller, opponent);
}

#[test]
fn polliwallop_reduces_generic_by_frogs_and_deals_double_power_without_fighting() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    put(&mut game, player, "Bellowing Crier", Zone::Battlefield);
    put(&mut game, opponent, "Bellowing Crier", Zone::Battlefield);
    let dealer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let victim = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Polliwallop", Zone::Hand);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 2];
    announce(
        &mut game,
        spell,
        &[Target::Card(dealer), Target::Card(victim)],
    );
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.object_view(dealer).damage, 0);
}

#[test]
fn hivespine_chooses_each_of_its_three_distinct_entry_modes() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let artifact = put(&mut game, opponent, "Candy Trail", Zone::Battlefield);
    game.create_tokens(opponent, Card::from_catalogue_id(5324), 1, false);
    assert!(!game.advance_entries());
    let token = game.controlled_token_ids(opponent).next().unwrap();
    let source = spell(&mut game, player, "Hivespine Wolverine", &[]);
    pass_pair(&mut game);
    for (mode, chosen) in [(0, bear), (1, token), (2, artifact)] {
        let mut branch = game.clone();
        act(&mut branch, Action::ChooseMode { mode });
        act(&mut branch, target(chosen));
        settle(&mut branch);
        match mode {
            0 => assert_eq!(branch.counter_count(bear, CounterKind::PlusOne), 1),
            1 => {
                assert!(!branch.on_battlefield(token));
                assert_eq!(branch.object_view(source).damage, 1);
            }
            2 => assert_eq!(branch.object_view(artifact).zone, Zone::Graveyard),
            _ => unreachable!(),
        }
    }
}

#[test]
fn inherited_red_removal_has_correct_damage_loot_and_exile_destinations() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let bear = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let artifact = put(&mut game, opponent, "Candy Trail", Zone::Battlefield);
    mode(&mut game, player, "Agate Assault", 0, Some(bear));
    assert_eq!(game.object_view(bear).zone, Zone::Exile);
    mode(&mut game, player, "Agate Assault", 1, Some(artifact));
    assert_eq!(game.object_view(artifact).zone, Zone::Exile);

    for (name, life) in [("Playful Shove", 19), ("Flame Lash", 15)] {
        game.state.players[player.index()].mana = [8; 6];
        let burn = put(&mut game, player, name, Zone::Hand);
        cast(&mut game, burn, Some(Target::Player(opponent)));
        settle(&mut game);
        assert_eq!(game.state.players[opponent.index()].life, life);
    }
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);

    put(&mut game, player, "Raccoon Rallier", Zone::Battlefield);
    let giant = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let discarded = put(&mut game, player, "Forest", Zone::Hand);
    spell(&mut game, player, "Take Out the Trash", &[giant]);
    pass_pair(&mut game);
    act(&mut game, Action::Discard { card: discarded });
    settle(&mut game);
    assert_eq!(game.object_view(giant).zone, Zone::Graveyard);
    assert_eq!(game.object_view(discarded).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

#[test]
fn inherited_tricks_have_distinct_boosts_protections_and_power_damage() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[bear as usize].tapped = true;
    spell(&mut game, player, "High Stride", &[bear]);
    settle(&mut game);
    assert!(!game.object_view(bear).tapped);
    assert_eq!(stats(&game, bear), (3, 5));
    assert!(game.keywords(bear).contains(Keywords::REACH));
    spell(&mut game, player, "Giant Growth", &[bear]);
    settle(&mut game);
    assert_eq!(stats(&game, bear), (6, 8));
    spell(&mut game, player, "Overprotect", &[bear]);
    settle(&mut game);
    assert_eq!(stats(&game, bear), (9, 11));
    assert!(
        game.keywords(bear).contains(
            Keywords::TRAMPLE
                .union(Keywords::HEXPROOF)
                .union(Keywords::INDESTRUCTIBLE)
        )
    );

    let mut bite = scenario(player);
    let dealer = put(&mut bite, player, "Grizzly Bears", Zone::Battlefield);
    let victim = put(&mut bite, opponent, "Quaketusk Boar", Zone::Battlefield);
    spell(&mut bite, player, "Rabid Gnaw", &[dealer, victim]);
    settle(&mut bite);
    assert_eq!(stats(&bite, dealer), (3, 2));
    assert_eq!(stats(&bite, victim), (5, 5));
    assert_eq!(bite.object_view(victim).damage, 3);
    spell(&mut bite, player, "Rabid Bite", &[dealer, victim]);
    settle(&mut bite);
    assert_eq!(bite.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(bite.object_view(dealer).damage, 0);
}

#[test]
fn inherited_auras_and_formation_resolve_each_attachment_or_mode() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let bear = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let blocker = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    spell(&mut game, player, "War Squeak", &[bear]);
    pass_pair(&mut game);
    act(&mut game, target(blocker));
    settle(&mut game);
    assert_eq!(stats(&game, bear), (3, 3));
    assert!(game.keywords(bear).contains(Keywords::HASTE));
    assert!(game.keywords(blocker).contains(Keywords::CANT_BLOCK));
    spell(&mut game, player, "Colossification", &[bear]);
    settle(&mut game);
    assert!(game.object_view(bear).tapped);
    assert_eq!(stats(&game, bear), (23, 23));

    let flier = put(&mut game, opponent, "Serra Angel", Zone::Battlefield);
    let enchantment = put(&mut game, opponent, "Doubling Season", Zone::Battlefield);
    mode(&mut game, player, "Pawpatch Formation", 0, Some(flier));
    assert_eq!(game.object_view(flier).zone, Zone::Graveyard);
    mode(
        &mut game,
        player,
        "Pawpatch Formation",
        1,
        Some(enchantment),
    );
    assert_eq!(game.object_view(enchantment).zone, Zone::Graveyard);
    mode(&mut game, player, "Pawpatch Formation", 2, None);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    assert!(
        game.controlled_token_ids(player)
            .any(|token| game.has_subtype(token, "Food"))
    );
}
