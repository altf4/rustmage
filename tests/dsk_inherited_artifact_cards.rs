//! Paid scenarios for the five already-admitted Duskmourn artifacts.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PLAYER: Player = Player::P0;
const OTHER: Player = Player::P1;

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn pay_if_needed(game: &mut Game) {
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

#[test]
fn bear_trap_can_be_cast_and_paid_for_on_the_opponents_turn() {
    let mut game = scenario(OTHER);
    let trap = put(&mut game, PLAYER, "Bear Trap", Zone::Hand);
    let creature = put(&mut game, OTHER, "Hill Giant", Zone::Battlefield);
    let land = put(&mut game, OTHER, "Forest", Zone::Battlefield);
    game.state.step = Step::BeginCombat;
    act(&mut game, Action::Pass);
    cast(&mut game, trap, None);
    settle(&mut game);
    assert_eq!(game.object_view(trap).zone, Zone::Battlefield);

    act(&mut game, Action::Pass);
    let mut unfunded = game.clone();
    unfunded.state.players[PLAYER.index()].mana = [0, 0, 0, 0, 0, 2];
    reject(&mut unfunded, activate(trap, 0));
    assert!(!unfunded.object_view(trap).tapped);

    game.state.players[PLAYER.index()].mana = [0, 0, 0, 0, 0, 3];
    act(&mut game, activate(trap, 0));
    reject(&mut game, target(land));
    act(&mut game, target(creature));
    pay(&mut game);
    assert_eq!(game.object_view(trap).zone, Zone::Graveyard);
    assert_eq!(game.state.players[PLAYER.index()].mana, [0; 6]);
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
}

#[test]
fn friendly_teddy_draws_for_both_players_only_after_an_actual_death() {
    let mut game = scenario(PLAYER);
    let teddy = put(&mut game, PLAYER, "Friendly Teddy", Zone::Hand);
    let removal = put(&mut game, PLAYER, "Murder", Zone::Hand);
    cast(&mut game, teddy, None);
    settle(&mut game);
    assert_eq!(stats(&game, teddy), (2, 2));
    assert_eq!(game.object_view(teddy).kinds & 17, 17);
    let mut replaced = game.clone();
    replaced.add_counter(teddy, CounterKind::Finality, 1);
    for branch in [&mut game, &mut replaced] {
        cast(branch, removal, Some(Target::Card(teddy)));
        settle(branch);
    }
    assert_eq!(game.object_view(teddy).zone, Zone::Graveyard);
    assert_eq!(replaced.object_view(teddy).zone, Zone::Exile);
    for seat in [PLAYER, OTHER] {
        assert_eq!(game.state.players[seat.index()].cards_drawn_this_turn, 1);
        assert_eq!(
            replaced.state.players[seat.index()].cards_drawn_this_turn,
            0
        );
    }
}

#[test]
fn ghost_vacuum_returns_only_its_linked_creatures_as_additive_spirits() {
    let mut game = scenario(PLAYER);
    let vacuum = put(&mut game, PLAYER, "Ghost Vacuum", Zone::Hand);
    let duplicate = put(&mut game, PLAYER, "Ghost Vacuum", Zone::Battlefield);
    let bear = put(&mut game, OTHER, "Grizzly Bears", Zone::Graveyard);
    let elf = put(&mut game, PLAYER, "Llanowar Elves", Zone::Graveyard);
    let spell = put(&mut game, PLAYER, "Opt", Zone::Graveyard);
    cast(&mut game, vacuum, None);
    settle(&mut game);

    for (source, card) in [(vacuum, bear), (vacuum, spell), (duplicate, elf)] {
        game.state.objects[source as usize].tapped = false;
        act(&mut game, activate(source, 0));
        act(&mut game, target(card));
        pay_if_needed(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(card).zone, Zone::Exile);
        assert!(game.object_view(source).tapped);
    }
    let exiled = game.object_ref(bear);
    game.state.objects[vacuum as usize].tapped = false;
    let mut combat = game.clone();
    combat.state.step = Step::BeginCombat;
    reject(&mut combat, activate(vacuum, 1));
    game.state.players[PLAYER.index()].mana = [0, 0, 0, 0, 0, 6];
    act(&mut game, activate(vacuum, 1));
    pay(&mut game);
    assert_eq!(game.object_view(vacuum).zone, Zone::Graveyard);
    assert_eq!(game.state.players[PLAYER.index()].mana, [0; 6]);
    settle(&mut game);
    let returned = game.object_view(bear);
    assert_eq!(returned.zone, Zone::Battlefield);
    assert_ne!(returned.reference, exiled);
    assert_eq!(returned.owner, OTHER);
    assert_eq!(returned.controller, PLAYER);
    assert!(returned.has_subtype("Bear") && returned.has_subtype("Spirit"));
    assert_eq!((returned.power, returned.toughness), (1, 1));
    assert!(returned.keywords.contains(Keywords::FLYING));
    assert_eq!(game.counter_count(bear, CounterKind::Flying), 1);
    assert_eq!(game.object_view(elf).zone, Zone::Exile);
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
}

#[test]
fn glimmerlight_creates_the_printed_token_and_equips_only_after_payment() {
    let mut game = scenario(PLAYER);
    let equipment = put(&mut game, PLAYER, "Glimmerlight", Zone::Hand);
    let bear = put(&mut game, PLAYER, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, OTHER, "Bear Cub", Zone::Battlefield);
    let new_id = game.state.objects.len() as CardId;
    cast(&mut game, equipment, None);
    settle(&mut game);
    let token = game.object_view(new_id);
    assert!(token.is_token() && token.has_subtype("Glimmer"));
    assert_eq!(
        (token.kinds, token.colors, token.power, token.toughness),
        (33, 1, 1, 1)
    );
    assert_eq!(game.object_view(equipment).attached_to, None);
    assert_eq!(stats(&game, bear), (2, 2));

    game.state.players[PLAYER.index()].mana = [0, 0, 0, 0, 0, 1];
    act(&mut game, activate(equipment, 0));
    reject(&mut game, target(enemy));
    act(&mut game, target(bear));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[PLAYER.index()].mana, [0; 6]);
    assert_eq!(
        game.object_view(equipment).attached_to,
        Some(game.object_ref(bear))
    );
    assert_eq!(stats(&game, bear), (3, 3));
}

#[test]
fn malevolent_chandelier_can_put_either_players_graveyard_card_on_its_owners_bottom() {
    let mut game = scenario(PLAYER);
    let chandelier = put(&mut game, PLAYER, "Malevolent Chandelier", Zone::Hand);
    let own = put(&mut game, PLAYER, "Forest", Zone::Graveyard);
    let theirs = put(&mut game, OTHER, "Shock", Zone::Graveyard);
    let outside_graveyard = put(&mut game, OTHER, "Shock", Zone::Exile);
    cast(&mut game, chandelier, None);
    settle(&mut game);
    assert_eq!(stats(&game, chandelier), (4, 4));
    assert!(game.keywords(chandelier).contains(Keywords::FLYING));

    let mut combat = game.clone();
    combat.state.step = Step::BeginCombat;
    reject(&mut combat, activate(chandelier, 0));
    for (owner, card) in [(PLAYER, own), (OTHER, theirs)] {
        game.state.players[PLAYER.index()].mana = [0, 0, 0, 0, 0, 2];
        act(&mut game, activate(chandelier, 0));
        reject(&mut game, target(outside_graveyard));
        act(&mut game, target(card));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.state.players[PLAYER.index()].mana, [0; 6]);
        assert_eq!(game.object_view(card).zone, Zone::Library);
        assert_eq!(
            game.state.players[owner.index()].library.first(),
            Some(&card)
        );
    }
}
