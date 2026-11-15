//! A permanent copied into a Saga uses copied lore chapters and sacrifice rules.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const SAGA: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 4 },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 4)],
        },
    ],
    ..CardRules::EMPTY
};
fn setup() -> (fixture_rules::Guard, Game, CardId) {
    let rules = fixture_rules::activate(&[(Card::from_name("Amok Time").unwrap(), &SAGA)]);
    let mut game = scenario(Player::P0);
    let original = put(&mut game, Player::P1, "Amok Time", Zone::Battlefield);
    (rules, game, original)
}

fn drive(game: &mut Game) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority && !game.state.pending_triggers.is_empty() {
            game.prepare_triggers();
        }
        if game.state.pending == Pending::Priority
            && game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("copied Saga resolution budget: {:?}", game.state.pending);
}

fn tick(game: &mut Game) {
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    drive(game);
}

fn assert_copied_chapters(game: &mut Game, copy: CardId, original: CardId) {
    let player = Player::P0;
    assert_eq!(
        game.object_view(copy).card,
        Card::from_name("Amok Time").unwrap()
    );
    assert_eq!(game.object_view(copy).zone, Zone::Battlefield);
    assert_eq!(game.counter_count(copy, CounterKind::Lore), 0);
    assert_eq!(game.counter_count(original, CounterKind::Lore), 0);
    let life = game.state.players[player.index()].life;
    for chapter in 1..=4 {
        tick(game);
        if chapter < 4 {
            assert_eq!(game.counter_count(copy, CounterKind::Lore), chapter as u64);
            assert_eq!(game.object_view(copy).zone, Zone::Battlefield);
        }
    }
    assert_eq!(game.state.players[player.index()].life, life + 5);
    assert_eq!(game.object_view(copy).zone, Zone::Graveyard);
    assert_eq!(game.object_view(original).zone, Zone::Battlefield);
}

#[test]
fn a_non_saga_becoming_a_saga_gains_lore_chapters_and_final_sacrifice() {
    let (_rules, mut game, original) = setup();
    let copy = put(&mut game, Player::P0, "Grizzly Bears", Zone::Battlefield);
    let copy_ref = game.object_ref(copy);
    game.apply_temporary_copy(
        copy_ref,
        CopyRecipe::new(Card::from_name("Amok Time").unwrap()),
        CopyDeadline::EndOfTurn,
    );
    assert_copied_chapters(&mut game, copy, original);
}
