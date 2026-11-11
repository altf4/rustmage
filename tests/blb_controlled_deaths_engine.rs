//! Per-controller creature death history used by a resolving draw amount.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SACRIFICE: Instruction = Instruction::ChooseObjects {
    kind: EffectChoiceKind::Sacrifice,
    players: PlayerSet::Each,
    filter: SacrificeFilter {
        kinds: 1,
        other: false,
        keywords: Keywords::NONE,
        source_only: false,
        nonland: false,
        tokens: None,
        artifact_enchantment_or_token: false,
        subtype: None,
    },
    amount: 1,
    optional_skip: None,
};
const DRAW: Instruction = Instruction::DrawScaled(
    PlayerSet::Controller,
    AmountValue::Stat(StatValue::ControlledCreaturesDiedThisTurn),
);
const DEATH_DRAW: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Draw for creatures you controlled that died this turn",
            targets: TargetPlan::NONE,
            program: &[DRAW],
        },
        SpellMode {
            text: "Each player sacrifices a creature twice, then draw for your deaths",
            targets: TargetPlan::NONE,
            program: &[SACRIFICE, SACRIFICE, DRAW],
        },
    ]),
    mode_counts: Some(&[1, 3]),
    ..CardRules::EMPTY
};

fn opt() -> Card {
    Card::from_name("Opt").unwrap()
}

fn sacrifice(game: &Game, card: CardId) -> Action {
    Action::Sacrifice {
        object: game.object_ref(card),
    }
}

fn start(game: &mut Game, player: Player, mode: u16) -> CardId {
    let spell = spawn(game, player, opt(), Zone::Hand);
    act(game, Action::AnnounceCast { card: spell });
    act(game, Action::ChooseMode { mode });
    pay(game);
    act(game, Action::Pass);
    act(game, Action::Pass);
    spell
}

fn death_count(game: &Game, player: Player, source: CardId) -> i128 {
    game.stat_value(
        StatValue::ControlledCreaturesDiedThisTurn,
        game.object_ref(source),
        player,
    )
}

#[test]
fn draw_counts_prior_and_current_turn_actual_deaths_under_the_right_controller() {
    let _rules = fixture_rules::activate(&[(opt(), &DEATH_DRAW)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let borrowed = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        let stolen = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        game.record_control(game.object_ref(borrowed), player, None);
        game.record_control(game.object_ref(stolen), opponent, None);
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Card(borrowed)));
        settle(&mut game);
        let shock = put(&mut game, player, "Shock", Zone::Hand);
        cast(&mut game, shock, Some(Target::Card(stolen)));
        settle(&mut game);
        assert_eq!(death_count(&game, player, borrowed), 1);
        assert_eq!(death_count(&game, opponent, stolen), 1);
        game.move_object(borrowed, Zone::Exile);
        game.move_object(stolen, Zone::Exile);

        let finality = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        game.add_counter(finality, CounterKind::Finality, 1);
        let their_first = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        let their_second = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        game.create_tokens(player, Card::from_catalogue_id(5254), 1, false);
        let token = game.controlled_token_ids(player).next().unwrap();
        let spell = start(&mut game, player, 1);
        assert_eq!(death_count(&game, player, spell), 1);

        let choose_finality = sacrifice(&game, finality);
        let choose_foreign = sacrifice(&game, their_first);
        reject(&mut game, choose_foreign.clone());
        act(&mut game, choose_finality);
        assert_eq!(game.object_view(finality).zone, Zone::Battlefield);
        act(&mut game, choose_foreign);
        assert_eq!(game.object_view(finality).zone, Zone::Exile);
        assert_eq!(death_count(&game, player, spell), 1);
        assert_eq!(death_count(&game, opponent, spell), 2);

        let choose_token = sacrifice(&game, token);
        act(&mut game, choose_token);
        let snapshot = game.snapshot();
        let choose_second = sacrifice(&game, their_second);
        let index = game
            .legal_actions()
            .iter()
            .position(|a| *a == choose_second)
            .unwrap();
        let mut indexed = game.clone();
        indexed
            .apply_index(opponent, game.revision(), index)
            .unwrap();
        act(&mut game, choose_second);
        assert_eq!(game.snapshot(), indexed.snapshot());
        for branch in [&mut game, &mut indexed] {
            assert_eq!(death_count(branch, player, spell), 2);
            assert_eq!(death_count(branch, opponent, spell), 3);
            assert_eq!(branch.state.creatures_died_this_turn, 5);
            assert_eq!(
                branch.state.players[player.index()].cards_drawn_this_turn,
                2
            );
            assert_eq!(
                branch.state.players[opponent.index()].cards_drawn_this_turn,
                0
            );
            assert!(branch.controlled_token_ids(player).next().is_none());
        }
        indexed.restore(&snapshot).unwrap();
        assert_eq!(indexed.snapshot(), snapshot);

        game.finish_cleanup();
        assert_eq!(death_count(&game, player, spell), 0);
        assert_eq!(death_count(&game, opponent, spell), 0);
    }
}

#[test]
fn a_controller_with_no_creature_deaths_draws_zero() {
    let _rules = fixture_rules::activate(&[(opt(), &DEATH_DRAW)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    game.move_object(creature, Zone::Graveyard);
    let spell = start(&mut game, player, 0);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(death_count(&game, player, spell), 0);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    assert_eq!(game.state.creatures_died_this_turn, 1);
}
