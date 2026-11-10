mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const FRONT: &str = "Brigid, Clachan's Heart";
const BACK: &str = "Brigid, Doun's Mind";
const FRONT_RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::ChooseOpposingCreatureForDeparture {
                other_power: -2,
                other_toughness: -2,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::ChosenCreatureLeaves,
            targets: TargetPlan::NONE,
            program: &[Instruction::TransformSource],
        },
    ],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const BACK_RULES: CardRules = CardRules {
    effect: Effect::Creature(8, 8),
    ..CardRules::EMPTY
};
const ANIMATE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 16,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[Instruction::AnimateTargets(AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Unchanged,
            base: Some((4, 4)),
            keywords: Keywords::NONE,
            duration: AnimationDuration::EndOfTurn,
        })],
    ),
    ..CardRules::EMPTY
};
const FORK: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Ability)),
        &[Instruction::CopyTargetAbility { group: 0 }],
    ),
    ..CardRules::EMPTY
};

fn fixture() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::from_name(FRONT).unwrap(), &FRONT_RULES),
        (Card::from_name(BACK).unwrap(), &BACK_RULES),
        (Card::from_name("Shore Up").unwrap(), &ANIMATE),
        (Card::from_name("Opt").unwrap(), &FORK),
    ])
}

fn pass_priority(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn enter(game: &mut Game, player: Player) -> CardId {
    let source = put(game, player, FRONT, Zone::Hand);
    cast(game, source, None);
    pass_priority(game);
    assert_eq!(game.object_view(source).zone, Zone::Battlefield);
    assert_eq!(game.state.stack.len(), 1);
    assert!(game.state.stack[0].targets.is_empty());
    source
}

fn choose_link(game: &mut Game, id: CardId) {
    act(
        game,
        Action::ChooseLinkedCreature {
            object: game.object_ref(id),
        },
    );
}

#[test]
fn the_owned_choice_and_departure_keep_the_exact_permanent_through_type_and_control_changes() {
    let _rules = fixture();
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let bike = put(&mut game, opponent, "Veloheart Bike", Zone::Battlefield);
    let giant = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let first_plains = put(&mut game, opponent, "Plains", Zone::Battlefield);
    let second_plains = put(&mut game, opponent, "Plains", Zone::Battlefield);
    act(&mut game, Action::Pass);
    let animate = put(&mut game, opponent, "Shore Up", Zone::Hand);
    cast(&mut game, animate, Some(Target::Card(bike)));
    settle(&mut game);
    let chosen = game.object_ref(bike);
    game.grant_keyword_effects(&[bike], chosen, Keywords::HEXPROOF);
    assert!(game.is_creature(bike));
    let source = enter(&mut game, player);
    let source_ref = game.object_ref(source);
    pass_priority(&mut game);
    let view = game.observe(opponent).effect_choice.unwrap();
    assert_eq!(view.candidates, vec![chosen, game.object_ref(giant)]);
    assert!(!view.optional);
    reject(&mut game, Action::DeclineEffectPayment);
    let action = Action::ChooseLinkedCreature { object: chosen };
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let saved = game.snapshot();
    let revision = game.revision();
    let mut indexed = game.clone();
    assert_eq!(game.apply(opponent, action.clone()), Err(Error::WrongActor));
    assert_eq!(game.snapshot(), saved);
    act(&mut game, action);
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    choose_link(&mut indexed, giant);
    assert_eq!(
        indexed.observe(opponent).chosen_creature_links[0].chosen,
        indexed.object_ref(giant)
    );
    let links = game.observe(player).chosen_creature_links;
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].source, source_ref);
    assert_eq!(links[0].chosen, chosen);
    assert_eq!(stats(&game, source), (4, 4));
    assert_eq!(stats(&game, bike), (4, 4));
    assert_eq!(stats(&game, giant), (8, 8));
    assert_eq!(game.object_view(bear).zone, Zone::Graveyard);

    game.state.step = Step::End;
    pass_priority(&mut game);
    assert_eq!(game.state.active, opponent);
    assert!(!game.is_creature(bike));
    assert_eq!(stats(&game, giant), (10, 10));
    game.change_controller(chosen, player);
    for land in [first_plains, second_plains] {
        act(&mut game, Action::TapMana { card: land });
    }
    let disenchant = put(&mut game, opponent, "Disenchant", Zone::Hand);
    cast(&mut game, disenchant, Some(Target::Card(bike)));
    pass_priority(&mut game);
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(game.state.stack[0].source.unwrap().object, source_ref);
    settle(&mut game);
    assert_eq!(game.object_view(source).name(), BACK);
    assert_eq!(game.object_ref(source), source_ref);
}

#[test]
fn no_eligible_choice_still_reduces_existing_other_creatures() {
    let _rules = fixture();
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let source = enter(&mut game, player);
    settle(&mut game);
    assert!(game.observe(player).chosen_creature_links.is_empty());
    assert_eq!(stats(&game, source), (4, 4));
    assert_eq!(stats(&game, first), (1, 1));
    let second = put(&mut game, player, "Hill Giant", Zone::Hand);
    cast(&mut game, second, None);
    settle(&mut game);
    assert_eq!(stats(&game, second), (3, 3));
}

#[test]
fn simultaneous_source_and_chosen_departures_queue_one_trigger_without_transforming() {
    let _rules = fixture();
    let player = Player::P0;
    let mut game = scenario(player);
    let chosen = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let source = enter(&mut game, player);
    let source_ref = game.object_ref(source);
    pass_priority(&mut game);
    choose_link(&mut game, chosen);
    let wipe = put(&mut game, player, "Day of Judgment", Zone::Hand);
    cast(&mut game, wipe, None);
    pass_priority(&mut game);
    assert_eq!(game.object_view(chosen).zone, Zone::Graveyard);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert_eq!(game.state.stack.len(), 1);
    assert_eq!(game.state.stack[0].source.unwrap().object, source_ref);
    assert!(game.observe(player).chosen_creature_links.is_empty());
    settle(&mut game);
    assert_eq!(game.object_view(source).name(), FRONT);
}

#[test]
fn a_copied_entry_choice_excludes_its_own_recipient_and_the_latest_choice_is_linked() {
    let _rules = fixture();
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let second = put(
        &mut game,
        player.other(),
        "Gigantosaurus",
        Zone::Battlefield,
    );
    let source = enter(&mut game, player);
    let trigger = game.state.stack[0].id;
    let copy = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, copy, Some(Target::Stack(trigger)));
    pass_priority(&mut game);
    if game.legal_actions().contains(&Action::FinishStackTargets) {
        act(&mut game, Action::FinishStackTargets);
    }
    pass_priority(&mut game);
    choose_link(&mut game, first);
    assert_eq!(stats(&game, first), (3, 3));
    assert_eq!(stats(&game, second), (8, 8));
    pass_priority(&mut game);
    choose_link(&mut game, second);
    assert_eq!(stats(&game, first), (1, 1));
    assert_eq!(stats(&game, second), (8, 8));
    assert_eq!(game.observe(player).chosen_creature_links.len(), 1);
    assert_eq!(
        game.observe(player).chosen_creature_links[0].chosen,
        game.object_ref(second)
    );
    for (creature, face) in [(first, FRONT), (second, BACK)] {
        let bounce = put(&mut game, player, "Unsummon", Zone::Hand);
        cast(&mut game, bounce, Some(Target::Card(creature)));
        settle(&mut game);
        assert_eq!(game.object_view(source).name(), face);
    }
}
