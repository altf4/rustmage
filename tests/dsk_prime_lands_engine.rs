//! An end-step guard based on actual land entry and a live prime count.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const END: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::LandEnteredAndPrimeLandCount,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensThenCounters {
            token: Card::SosFractalToken,
            amount: 1,
            kind: CounterKind::PlusOne,
            counters: AmountValue::Stat(StatValue::CONTROLLER_LANDS),
        }],
    }],
    ..CardRules::EMPTY
};

const REMOVE_LAND: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(rustmage_core::cards::authored::object(2))),
        &[Instruction::Destroy],
    ),
    ..CardRules::EMPTY
};

fn end_with_lands(player: Player, count: u32, land_entered: bool) -> (Game, CardId, CardId) {
    let mut game = scenario(player);
    put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let island = put(&mut game, player, "Island", Zone::Battlefield);
    for _ in 2..count {
        put(&mut game, player, "Forest", Zone::Battlefield);
    }
    let last = put(
        &mut game,
        player,
        "Forest",
        if land_entered {
            Zone::Hand
        } else {
            Zone::Battlefield
        },
    );
    if land_entered {
        act(&mut game, Action::PlayLand { card: last });
    }
    game.begin_step(Step::End);
    game.prepare_triggers();
    (game, island, last)
}

#[test]
fn both_trigger_and_resolution_require_a_prime_count_and_an_actual_land_entry() {
    let _scope = fixture_rules::activate(&[
        (Card::GrizzlyBears, &END),
        (Card::from_name("Opt").unwrap(), &REMOVE_LAND),
    ]);
    let player = Player::P1;
    let (composite, _, _) = end_with_lands(player, 36, true);
    assert!(composite.state.stack.is_empty());
    let (no_entry, _, _) = end_with_lands(player, 37, false);
    assert!(no_entry.state.stack.is_empty());

    let (mut game, island, last) = end_with_lands(player, 37, true);
    assert_eq!(game.state.stack.len(), 1);
    let mut shrunk = game.clone();
    act(&mut shrunk, Action::TapMana { card: island });
    let spell = put(&mut shrunk, player, "Opt", Zone::Hand);
    cast(&mut shrunk, spell, Some(Target::Card(last)));
    settle(&mut shrunk);
    assert_eq!(shrunk.object_view(last).zone, Zone::Graveyard);
    assert!(
        !shrunk
            .observe(player)
            .objects
            .iter()
            .any(|view| view.card == Card::SosFractalToken)
    );

    let saved = game.snapshot();
    let mut indexed = game.clone();
    let pass = game
        .legal_actions()
        .iter()
        .position(|action| *action == Action::Pass)
        .unwrap();
    indexed.apply_index(player, game.revision(), pass).unwrap();
    act(&mut game, Action::Pass);
    settle(&mut indexed);
    settle(&mut game);
    assert_eq!(indexed.snapshot(), game.snapshot());
    indexed.restore(&saved).unwrap();
    let view = game.observe(player).objects;
    let token = view
        .iter()
        .find(|view| view.card == Card::SosFractalToken)
        .unwrap();
    assert_eq!((token.power, token.toughness), (37, 37));
    assert_eq!(game.counter_count(token.id, CounterKind::PlusOne), 37);
}
