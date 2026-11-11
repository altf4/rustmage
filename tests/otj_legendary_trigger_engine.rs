//! Legendary-creature trigger multiplicity follows the actual source.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PROVIDER: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AdditionalTriggers {
            scope: AdditionalTriggerScope::ControlledLegendaryCreature,
            condition: StatValue::Fixed(1),
        }],
    }],
    ..CardRules::EMPTY
};
const END: TriggeredAbility = TriggeredAbility {
    condition: TriggerCondition::AtStep {
        step: TriggerStep::End,
        relation: CastRelation::Any,
        guard: StepGuard::Always,
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
};
const CREATURE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        END,
        TriggeredAbility {
            condition: TriggerCondition::SelfCreatureDies,
            ..END
        },
    ],
    ..CardRules::EMPTY
};
const LEGENDARY_NONCREATURE: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[END],
    ..CardRules::EMPTY
};

fn settle_triggers(game: &mut Game) {
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
    settle(game);
}

#[test]
fn only_legendary_creature_sources_controlled_by_active_providers_get_extras() {
    let legendary_creature = Card::from_name("Wylie Duke, Atiin Hero").unwrap();
    let legendary_noncreature = Card::from_name("The Wind Crystal").unwrap();
    let provider = Card::from_name("Doubling Season").unwrap();
    let _rules = fixture_rules::activate(&[
        (provider, &PROVIDER),
        (legendary_creature, &CREATURE),
        (Card::GrizzlyBears, &CREATURE),
        (legendary_noncreature, &LEGENDARY_NONCREATURE),
    ]);
    let player = Player::P0;
    let mut game = scenario(player);
    let suppressed = spawn(&mut game, player, provider, Zone::Battlefield);
    spawn(&mut game, player, provider, Zone::Battlefield);
    spawn(&mut game, player, provider, Zone::Battlefield);
    spawn(&mut game, player, legendary_creature, Zone::Battlefield);
    spawn(
        &mut game,
        player.other(),
        legendary_creature,
        Zone::Battlefield,
    );
    spawn(&mut game, player, Card::GrizzlyBears, Zone::Battlefield);
    spawn(&mut game, player, legendary_noncreature, Zone::Battlefield);
    let suppressed_ref = game.object_ref(suppressed);
    game.remove_abilities_exact(
        &[suppressed_ref],
        suppressed_ref,
        player,
        AnimationDuration::EndOfTurn,
    );
    let life = game.state.players[player.index()].life;
    let enemy_life = game.state.players[player.other().index()].life;
    game.state.step = Step::Main2;
    game.give_priority();
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.state.step, Step::End);
    settle_triggers(&mut game);
    assert_eq!(game.state.players[player.index()].life, life + 5);
    assert_eq!(
        game.state.players[player.other().index()].life,
        enemy_life + 1
    );
}

#[test]
fn the_departing_legendary_creatures_own_trigger_uses_its_last_characteristics() {
    let legendary_creature = Card::from_name("Wylie Duke, Atiin Hero").unwrap();
    let provider = Card::from_name("Doubling Season").unwrap();
    let _rules = fixture_rules::activate(&[(provider, &PROVIDER), (legendary_creature, &CREATURE)]);
    let player = Player::P0;
    let mut game = scenario(player);
    spawn(&mut game, player, provider, Zone::Battlefield);
    let dying = spawn(&mut game, player, legendary_creature, Zone::Battlefield);
    let removal = put(&mut game, player, "Murder", Zone::Hand);
    let life = game.state.players[player.index()].life;

    cast(&mut game, removal, Some(Target::Card(dying)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.object_view(dying).zone, Zone::Graveyard);
    settle_triggers(&mut game);
    assert_eq!(game.state.players[player.index()].life, life + 2);
}
