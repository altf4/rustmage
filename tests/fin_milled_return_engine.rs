//! A post-mill choice records the actual public card returned to hand.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const GREETER: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(4)),
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::Kinds(2),
                count: MilledCount::Range {
                    minimum: 0,
                    maximum: 1,
                },
                destination: MilledDestination::HandAndCountSubtype("Town"),
            }),
            Instruction::IfAmountAtLeast {
                value: AmountValue::MovedCount,
                minimum: 1,
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 2),
        ],
    }],
    ..CardRules::EMPTY
};

fn await_greeter(game: &mut Game, player: Player) {
    let source = spawn(game, player, Card::GrizzlyBears, Zone::Hand);
    cast(game, source, None);
    for _ in 0..4 {
        act(game, Action::Pass);
    }
    assert_eq!(game.state.pending, Pending::MilledSelection { player });
}

#[test]
fn only_an_exact_milled_town_that_enters_hand_pays_the_life_bonus() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &GREETER)]);
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let deep_town = put(&mut game, player, "Vector, Imperial Capital", Zone::Library);
    let old_town = put(
        &mut game,
        player,
        "Vector, Imperial Capital",
        Zone::Graveyard,
    );
    let spell = put(&mut game, player, "Shock", Zone::Library);
    let forest = put(&mut game, player, "Forest", Zone::Library);
    let town = put(&mut game, player, "Vector, Imperial Capital", Zone::Library);
    put(&mut game, player, "Swiftfoot Boots", Zone::Library);

    await_greeter(&mut game, player);
    let exact = game.object_ref(town);
    let forest = game.object_ref(forest);
    let choice = game.observe(player).milled_choice.unwrap();
    assert_eq!(choice.candidates, [exact, forest]);
    for invalid in [spell, deep_town, old_town] {
        let object = game.object_ref(invalid);
        reject(&mut game, Action::SelectMilled { object });
    }
    let action = Action::SelectMilled { object: exact };
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let revision = game.revision();
    let saved = game.snapshot();
    let mut raw = game.clone();
    act(&mut raw, action.clone());
    game.apply_index(player, revision, index).unwrap();
    for branch in [&raw, &game] {
        assert_eq!(branch.object_view(town).zone, Zone::Hand);
        assert!(
            branch
                .observe(opponent)
                .known_hand
                .contains(&branch.object_ref(town))
        );
        assert_eq!(branch.state.players[player.index()].life, 22);
    }
    assert_eq!(raw.snapshot(), game.snapshot());

    raw.restore(&saved).unwrap();
    act(&mut raw, Action::SelectMilled { object: forest });
    assert_eq!(raw.object_view(forest.card).zone, Zone::Hand);
    assert_eq!(raw.object_view(town).zone, Zone::Graveyard);
    assert_eq!(raw.state.players[player.index()].life, 20);

    raw.restore(&saved).unwrap();
    act(&mut raw, Action::FinishMilledSelection);
    assert_eq!(raw.object_view(town).zone, Zone::Graveyard);
    assert_eq!(raw.state.players[player.index()].life, 20);

    raw.restore(&saved).unwrap();
    raw.move_object(town, Zone::Exile);
    reject(&mut raw, action);
    act(&mut raw, Action::FinishMilledSelection);
    assert_eq!(raw.object_view(town).zone, Zone::Exile);
    assert_eq!(raw.state.players[player.index()].life, 20);
}

#[test]
fn a_public_mill_replacement_keeps_the_actual_town_eligible() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &GREETER)]);
    let player = Player::P0;
    let mut game = scenario(player);
    put(
        &mut game,
        player.other(),
        "Rest in Peace",
        Zone::Battlefield,
    );
    let town = put(&mut game, player, "Vector, Imperial Capital", Zone::Library);
    await_greeter(&mut game, player);
    assert_eq!(game.object_view(town).zone, Zone::Exile);
    let object = game.object_ref(town);
    act(&mut game, Action::SelectMilled { object });
    assert_eq!(game.object_view(town).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].life, 22);
}
