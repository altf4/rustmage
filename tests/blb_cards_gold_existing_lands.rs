//! Inherited Bloomburrow land choices, mana, and token costs.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn announce(game: &mut Game, card: CardId, ability: u16) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
}

fn mana(card: CardId, ability: u16, payment: Mana, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability,
        payment,
        production,
        counter_payment: Counters::default(),
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

const ZERO: Mana = [0; 6];
const COLORLESS: Mana = [0, 0, 0, 0, 0, 1];
const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const RED: Mana = [0, 0, 0, 1, 0, 0];

#[test]
fn fabled_passage_untaps_only_the_found_basic_if_it_is_the_fourth_land() {
    let player = Player::P0;
    for lands in [2, 3] {
        let mut game = scenario(player);
        for _ in 0..lands {
            let land = put(&mut game, player, "Forest", Zone::Battlefield);
            game.state.objects[land as usize].tapped = true;
        }
        let target = put(&mut game, player, "Island", Zone::Library);
        let nonbasic = put(&mut game, player, "Hidden Grotto", Zone::Library);
        let passage = put(&mut game, player, "Fabled Passage", Zone::Hand);
        act(&mut game, Action::PlayLand { card: passage });
        assert!(!game.object_view(passage).tapped);
        announce(&mut game, passage, 0);
        if matches!(game.state.pending, Pending::PayCosts { .. }) {
            pay(&mut game);
        }
        assert_eq!(game.object_view(passage).zone, Zone::Graveyard);
        pass_pair(&mut game);
        reject(&mut game, Action::ChooseSearch { card: nonbasic });
        act(&mut game, Action::ChooseSearch { card: target });
        if game.legal_actions().contains(&Action::FinishSearch) {
            act(&mut game, Action::FinishSearch);
        }
        assert_eq!(game.object_view(target).zone, Zone::Battlefield);
        assert_eq!(game.object_view(target).tapped, lands == 2);
        assert_eq!(game.object_view(nonbasic).zone, Zone::Library);
    }
}

#[test]
fn fountainport_has_four_distinct_cost_and_reward_branches() {
    let player = Player::P1;
    let mut game = scenario(player);
    let port = put(&mut game, player, "Fountainport", Zone::Battlefield);
    let physical = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let token = game.state.objects.len() as CardId;
    game.create_tokens(player, Card::from_catalogue_id(5304), 1, false);

    let mut colorless = game.clone();
    colorless.state.players[player.index()].mana = ZERO;
    act(&mut colorless, mana(port, 0, ZERO, COLORLESS));
    assert!(colorless.object_view(port).tapped);
    assert_eq!(colorless.state.players[player.index()].mana, COLORLESS);

    let mut draw = game.clone();
    announce(&mut draw, port, 1);
    let invalid = Action::ChooseAdditionalCost {
        object: draw.object_ref(physical),
    };
    reject(&mut draw, invalid);
    let sacrifice = Action::ChooseAdditionalCost {
        object: draw.object_ref(token),
    };
    act(&mut draw, sacrifice);
    act(
        &mut draw,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 2],
        },
    );
    settle(&mut draw);
    assert!(draw.object_view(port).tapped);
    assert_ne!(draw.object_view(token).zone, Zone::Battlefield);
    assert_eq!(draw.state.players[player.index()].cards_drawn_this_turn, 1);

    for (ability, generic, life, subtype) in [(2, 3, 19, "Fish"), (3, 4, 20, "Treasure")] {
        let mut branch = game.clone();
        let created = branch.state.objects.len() as CardId;
        announce(&mut branch, port, ability);
        act(
            &mut branch,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 0, generic],
            },
        );
        settle(&mut branch);
        assert!(branch.object_view(port).tapped);
        assert!(branch.object_view(created).is_token() && branch.has_subtype(created, subtype));
        assert!(!branch.object_view(created).tapped);
        assert_eq!(branch.state.players[player.index()].life, life);
    }
}

#[test]
fn hidden_grotto_surveils_privately_then_offers_colorless_or_paid_colored_mana() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Hidden Grotto", Zone::Hand);
    let top = put(&mut game, player, "Plains", Zone::Library);
    act(&mut game, Action::PlayLand { card: land });
    pass_pair(&mut game);
    assert!(!game.object_view(land).tapped);
    assert!(game.observe(player).library_choice.unwrap().surveil);
    assert!(game.observe(player.other()).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Graveyard,
        },
    );
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);

    let mut colorless = game.clone();
    colorless.state.players[player.index()].mana = ZERO;
    act(&mut colorless, mana(land, 0, ZERO, COLORLESS));
    assert_eq!(colorless.state.players[player.index()].mana, COLORLESS);
    game.state.players[player.index()].mana = COLORLESS;
    reject(&mut game, mana(land, 1, COLORLESS, COLORLESS));
    act(&mut game, mana(land, 1, COLORLESS, RED));
    assert_eq!(game.state.players[player.index()].mana, RED);
}

#[test]
fn uncharted_haven_enters_tapped_and_uses_the_color_of_its_current_incarnation() {
    let player = Player::P1;
    let mut game = scenario(player);
    game.state.players[player.index()].mana = ZERO;
    let haven = put(&mut game, player, "Uncharted Haven", Zone::Hand);
    act(&mut game, Action::PlayLand { card: haven });
    for (color, production, wrong) in [
        (ChoiceColor::Blue, BLUE, RED),
        (ChoiceColor::Red, RED, BLUE),
    ] {
        let entry = Action::ChooseEntryValue {
            value: EntryChoiceValue::Color(color),
        };
        act(&mut game, entry);
        assert!(game.object_view(haven).tapped);
        reject(&mut game, mana(haven, 0, ZERO, production));
        game.state.objects[haven as usize].tapped = false;
        reject(&mut game, mana(haven, 0, ZERO, wrong));
        act(&mut game, mana(haven, 0, ZERO, production));
        assert_eq!(game.state.players[player.index()].mana, production);
        if color == ChoiceColor::Blue {
            game.state.players[player.index()].mana = ZERO;
            let old = game.object_ref(haven);
            game.move_object(haven, Zone::Exile);
            let return_haven = game.entry_move(haven, Zone::Battlefield, false);
            game.stage_entries(vec![return_haven], None);
            assert!(game.advance_entries());
            assert!(matches!(game.state.pending, Pending::EntryValue { .. }));
            assert_ne!(game.object_ref(haven), old);
        }
    }
}
