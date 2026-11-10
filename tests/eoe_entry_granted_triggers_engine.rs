//! Source-independent, exact trigger grants after a private hand placement.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const PLACEMENT: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CaptureEntryResult,
            Instruction::PutHandFiltered {
                filter: SearchFilter::Kinds(1 | 16),
                skip: 2,
            },
            Instruction::GrantEntryResultKeywords {
                keywords: Keywords::HASTE,
                permanent: true,
            },
            Instruction::GrantEntryResultTriggers(&[0, 1]),
        ],
    ),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::SelfLeaves),
            targets: TargetPlan::NONE,
            program: &[Instruction::DamageEachCreatureByDepartureManaValue],
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            }),
            targets: TargetPlan::NONE,
            program: &[Instruction::SacrificeSource],
        },
    ],
    ..CardRules::EMPTY
};

fn resolve_top(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn start_placement(game: &mut Game, player: Player) {
    let spell = put(game, player, "Divination", Zone::Hand);
    cast(game, spell, None);
    resolve_top(game);
}

#[test]
fn private_placement_grants_to_one_incarnation_and_departure_uses_copied_mana_value() {
    let _fixture = fixture_rules::activate(&[(Card::from_name("Divination").unwrap(), &PLACEMENT)]);
    let p = Player::P0;
    let mut game = scenario(p);
    let recipient = put(&mut game, p, "Grizzly Bears", Zone::Hand);
    let land = put(&mut game, p, "Forest", Zone::Hand);
    let small = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    let large = put(&mut game, p.other(), "Gigantosaurus", Zone::Battlefield);
    start_placement(&mut game, p);
    let object = game.object_ref(recipient);
    assert_eq!(game.observe(p).effect_choice.unwrap().candidates, [object]);
    assert!(game.observe(p.other()).effect_choice.is_none());
    let land = game.object_ref(land);
    reject(&mut game, Action::PutHandPermanent { object: land });
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineHandEntry);
    assert_eq!(declined.object_view(recipient).zone, Zone::Hand);
    assert!(declined.observe(p).temporary_abilities.is_empty());
    act(&mut game, Action::PutHandPermanent { object });
    settle(&mut game);
    let entered = game.object_ref(recipient);
    assert!(
        game.object_view(recipient)
            .keywords
            .contains(Keywords::HASTE)
    );
    let grants = game.observe(p).temporary_abilities;
    assert_eq!(grants.len(), 2);
    assert!(
        grants
            .iter()
            .all(|grant| grant.recipient == entered && grant.permanent)
    );

    let blueprint = game.copiable_recipe(game.object_ref(large));
    game.apply_temporary_copy(entered, blueprint, CopyDeadline::EndOfTurn);
    assert_eq!(game.object_mana_value(recipient), 5);
    let bounce = put(&mut game, p, "Unsummon", Zone::Hand);
    cast(&mut game, bounce, Some(Target::Card(recipient)));
    resolve_top(&mut game);
    assert_eq!(game.object_view(recipient).zone, Zone::Hand);
    assert_eq!(game.object_mana_value(recipient), 2);
    assert!(game.observe(p).temporary_abilities.is_empty());
    let packet = game.state.stack.last().unwrap();
    assert_eq!(packet.source.unwrap().object, entered);
    assert_eq!(
        packet.event_object.as_ref().unwrap().facts,
        EventObjectFacts::DepartureManaValue(5)
    );

    let saved = game.snapshot();
    game.move_object(recipient, Zone::Battlefield);
    assert!(
        !game
            .object_view(recipient)
            .keywords
            .contains(Keywords::HASTE)
    );
    settle(&mut game);
    assert_eq!(game.object_view(small).zone, Zone::Graveyard);
    assert_eq!(game.object_view(recipient).zone, Zone::Graveyard);
    assert_eq!(game.object_view(large).damage, 5);
    game.restore(&saved).unwrap();
    settle(&mut game);
    assert_eq!(game.object_view(recipient).zone, Zone::Hand);
    assert_eq!(game.object_view(large).damage, 5);
}

#[test]
fn artifact_grant_uses_the_live_controller_on_their_later_end_step() {
    let _fixture = fixture_rules::activate(&[(Card::from_name("Divination").unwrap(), &PLACEMENT)]);
    let p = Player::P1;
    let mut game = scenario(p);
    let artifact = put(&mut game, p, "Prophetic Prism", Zone::Hand);
    let survivor = put(&mut game, p, "Hill Giant", Zone::Battlefield);
    start_placement(&mut game, p);
    let object = game.object_ref(artifact);
    act(&mut game, Action::PutHandPermanent { object });
    settle(&mut game);
    let entrant = game.object_ref(artifact);
    game.change_controller(entrant, p.other());

    game.state.step = Step::Main2;
    resolve_top(&mut game);
    assert_eq!(game.state.step, Step::End);
    assert!(game.state.stack.is_empty());
    assert_eq!(game.object_view(artifact).zone, Zone::Battlefield);
    game.state.active = p.other();
    game.state.priority = p.other();
    game.state.turn += 1;
    game.state.step = Step::Main2;
    resolve_top(&mut game);
    let sacrifice = game.state.stack.last().unwrap();
    assert_eq!(sacrifice.kind, StackKind::TriggeredAbility { ability: 1 });
    assert_eq!(sacrifice.controller, p.other());
    assert_eq!(sacrifice.source.unwrap().object, entrant);
    resolve_top(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    let damage = game.state.stack.last().unwrap();
    assert_eq!(damage.kind, StackKind::TriggeredAbility { ability: 0 });
    assert_eq!(damage.controller, p.other());
    settle(&mut game);
    assert_eq!(game.object_view(survivor).damage, 2);
}
