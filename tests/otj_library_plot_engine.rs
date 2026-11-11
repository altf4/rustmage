//! Live private library access, distinct printed/granted Plot prices, and exact pending X.
mod support;
use std::num::NonZeroU32;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const LIBRARY_PLOT: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::LookAtLibraryTop],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::PlotFromLibraryTop],
        },
    ],
    ..CardRules::EMPTY
};

fn fixture_card() -> Card {
    Card::from_name("Short Bow").unwrap()
}

fn top_plot(game: &Game, id: CardId, colored: Mana, generic: u32, x_symbols: u32) -> Action {
    Action::AnnounceLibraryTopPlot {
        object: game.object_ref(id),
        cost: ManaCost { colored, generic },
        x_symbols: NonZeroU32::new(x_symbols),
    }
}

fn pay_plot(game: &mut Game) {
    choose(game, |action| matches!(action, Action::PayPlot { .. }));
}

#[test]
fn library_plot_prices_are_separate_and_hand_reduction_does_not_apply() {
    let _rules = fixture_rules::activate(&[(fixture_card(), &LIBRARY_PLOT)]);
    let player = Player::P1;
    let mut game = scenario(player);
    put(&mut game, player, "Short Bow", Zone::Battlefield);
    put(
        &mut game,
        player,
        "Doc Aurlock, Grizzled Genius",
        Zone::Battlefield,
    );
    let hidden_next = put(&mut game, player, "Island", Zone::Library);
    let library = put(&mut game, player, "Freestrider Commando", Zone::Library);
    let hand = put(&mut game, player, "Freestrider Commando", Zone::Hand);
    let green = [0, 0, 0, 0, 1, 0];
    let printed_plot = top_plot(&game, library, green, 3, 0);
    let mana_cost_plot = top_plot(&game, library, green, 2, 0);
    let hand_plot = Action::AnnouncePlot { card: hand };

    let view = game.observe(player).known_library_top[player.index()].unwrap();
    assert_eq!(view.object, game.object_ref(library));
    assert!(game.observe(player.other()).known_library_top[player.index()].is_none());
    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 1];
    assert!(game.legal_actions().contains(&hand_plot));
    reject(&mut game, printed_plot.clone());
    reject(&mut game, mana_cost_plot.clone());

    game.state.players[player.index()].mana = [0, 0, 0, 0, 1, 3];
    let checkpoint = game.snapshot();
    for (announce, left) in [(printed_plot, 0), (mana_cost_plot.clone(), 1)] {
        let mut branch = game.clone();
        let mut indexed = game.clone();
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == announce)
            .unwrap();
        act(&mut branch, announce);
        indexed.apply_index(player, game.revision(), index).unwrap();
        assert_eq!(branch.snapshot(), indexed.snapshot());
        let payment = branch.observe(player).plot_payment.unwrap();
        assert!(payment.from_library);
        assert_eq!(payment.object, game.object_ref(library));
        assert!(branch.observe(player.other()).plot_payment.is_none());
        assert_eq!(
            branch.observe(player).known_library_top[player.index()]
                .unwrap()
                .object,
            payment.object,
        );

        pay_plot(&mut branch);
        pay_plot(&mut indexed);
        assert_eq!(branch.snapshot(), indexed.snapshot());
        assert_eq!(branch.state.players[player.index()].mana[5], left);
        assert_eq!(branch.object_view(library).zone, Zone::Exile);
        assert_eq!(
            branch.observe(player).known_library_top[player.index()]
                .unwrap()
                .object,
            branch.object_ref(hidden_next),
        );
        assert!(branch.observe(player.other()).known_library_top[player.index()].is_none());
        assert_eq!(
            branch.state.players[player.index()].spells_cast_this_turn,
            0
        );
        indexed.restore(&checkpoint).unwrap();
        assert_eq!(indexed.snapshot(), game.snapshot());
    }

    // The old announcement cannot be used once its card is no longer the top.
    put(&mut game, player, "Forest", Zone::Library);
    reject(&mut game, mana_cost_plot);
    assert!(
        !game
            .legal_actions()
            .iter()
            .any(|a| matches!(a, Action::AnnounceLibraryTopPlot { .. }))
    );
}

#[test]
fn library_plot_uses_actual_mana_symbols_and_ignores_an_inset_face() {
    let _rules = fixture_rules::activate(&[(fixture_card(), &LIBRARY_PLOT)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Short Bow", Zone::Battlefield);
    let hybrid = put(&mut game, player, "Rakshasa's Bargain", Zone::Library);
    let fully_colored = top_plot(&game, hybrid, [0, 1, 1, 0, 1, 0], 0, 0);
    let fully_generic = top_plot(&game, hybrid, [0; 6], 6, 0);
    assert!(game.legal_actions().contains(&fully_colored));
    assert!(game.legal_actions().contains(&fully_generic));
    game.state.players[player.index()].mana = [0, 1, 1, 0, 1, 0];
    reject(&mut game, fully_generic);
    act(&mut game, fully_colored);
    pay_plot(&mut game);

    let adventurer = put(
        &mut game,
        player,
        "Elusive Otter // Grove's Bounty",
        Zone::Library,
    );
    let front_cost = top_plot(&game, adventurer, [0, 1, 0, 0, 0, 0], 0, 0);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    act(&mut game, front_cost);
    pay_plot(&mut game);
    game.state.turn += 1;
    assert!(
        game.legal_actions()
            .contains(&Action::AnnouncePlotted { card: adventurer })
    );
    assert!(game.legal_actions().contains(&Action::AnnouncePlottedFace {
        card: adventurer,
        face: CastFace::Adventure,
    }));
}

#[test]
fn choosing_x_for_the_special_action_does_not_carry_it_into_the_later_spell() {
    let _rules = fixture_rules::activate(&[(fixture_card(), &LIBRARY_PLOT)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Short Bow", Zone::Battlefield);
    let spring = put(&mut game, player, "Mind Spring", Zone::Library);
    game.state.players[player.index()].mana = [0, 2, 0, 0, 0, 3];
    let plot = top_plot(&game, spring, [0, 2, 0, 0, 0, 0], 0, 1);
    act(&mut game, plot);
    let choice = game.observe(player).x_choice.unwrap();
    assert_eq!(
        choice.intervals,
        vec![XInterval {
            minimum: 0,
            maximum: 3
        }]
    );
    assert!(game.observe(player.other()).x_choice.is_none());
    reject(&mut game, Action::ChooseX { value: 4 });
    act(&mut game, Action::ChooseX { value: 3 });
    pay_plot(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);

    game.state.turn += 1;
    act(&mut game, Action::AnnouncePlotted { card: spring });
    assert!(game.x_choice().is_none());
    pay(&mut game);
    assert_eq!(game.state.stack.last().unwrap().x, Some(0));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
}
