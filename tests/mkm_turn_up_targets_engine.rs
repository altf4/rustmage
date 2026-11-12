//! One optional mixed-zone target group bounded by the actual turn-up X.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::{cards::*, engine::*};
use support::*;

const DISGUISE: ManaCost = ManaCost {
    colored: [1, 0, 0, 0, 0, 0],
    generic: 3,
};
const TARGETS: TargetPlan = TargetPlan::Single(TargetSpec {
    rule: TargetRule::Union(&[
        TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: -1,
            other: true,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        },
        TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        },
    ]),
    min: 0,
    max: usize::MAX,
});
const VARIABLE: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfTurnedFaceUp,
            targets: TARGETS,
            program: &[
                Instruction::UpToTargets(AmountValue::ChosenX),
                Instruction::ExileTargetsLinkedToSource,
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfLeaves,
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnLinkedExileToHand],
        },
    ],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec {
            mana: DISGUISE,
            alternatives: &[],
            modifier: DisguiseCostModifier::X(std::num::NonZeroU8::MIN),
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const FORCE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::TurnTargetFaceUp],
    ),
    ..CardRules::EMPTY
};

fn named(name: &str) -> Card {
    Card::from_name(name).unwrap()
}

fn select(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn pay_turn_up(game: &mut Game) {
    choose(game, |action| matches!(action, Action::PayFaceUp { .. }));
}

fn no_targets(game: &mut Game, source: ObjectRef, candidate: CardId) {
    assert!(!game.is_face_down(source));
    let trigger = game.state.stack.last().unwrap();
    assert_eq!(trigger.x, Some(0));
    assert!(trigger.targets.is_empty());
    reject(game, select(candidate));
    settle(game);
    assert_eq!(game.object_view(candidate).zone, Zone::Battlefield);
}

#[test]
fn paid_x_bounds_one_optional_group_and_the_link_returns_the_exact_cohort() {
    let source = named("Grizzly Bears");
    let _rules = fixture_rules::activate(&[(source, &VARIABLE)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let first = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, opponent, "Healer's Hawk", Zone::Graveyard);
    let third = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let ineligible = put(&mut game, player, "Shock", Zone::Graveyard);
    let hidden = spawn(&mut game, player, source, Zone::Hand);
    act(&mut game, Action::AnnounceDisguise { card: hidden });
    pay(&mut game);
    settle(&mut game);
    let object = game.object_ref(hidden);
    act(
        &mut game,
        Action::BeginFaceUpX {
            object,
            cost: DISGUISE,
            x_symbols: std::num::NonZeroU8::MIN,
        },
    );
    act(&mut game, Action::ChooseX { value: 2 });
    pay_turn_up(&mut game);

    let mut empty = game.clone();
    act(&mut empty, Action::FinishTargets);
    settle(&mut empty);
    assert_eq!(empty.object_view(first).zone, Zone::Battlefield);
    assert_eq!(empty.object_view(second).zone, Zone::Graveyard);
    reject(&mut game, select(hidden));
    reject(&mut game, select(ineligible));
    act(&mut game, select(first));
    reject(&mut game, select(first));
    act(&mut game, select(second));
    reject(&mut game, select(third));
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(&mut game, Action::FinishTargets);
    }
    let trigger = game.state.stack.last().unwrap();
    assert_eq!(trigger.x, Some(2));
    assert_eq!(trigger.targets.len(), 2);
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Exile);
    assert_eq!(game.object_view(second).zone, Zone::Exile);
    assert_eq!(game.object_view(third).zone, Zone::Graveyard);

    game.move_object(hidden, Zone::Graveyard);
    game.prepare_triggers();
    settle(&mut game);
    for card in [first, second] {
        let returned = game.object_view(card);
        assert_eq!(returned.zone, Zone::Hand);
        assert_eq!(returned.owner, opponent);
    }
}

#[test]
fn forced_and_creature_mana_turn_ups_each_offer_zero_targets() {
    let source = named("Grizzly Bears");
    let forcing_spell = named("Opt");
    let _rules = fixture_rules::activate(&[(source, &VARIABLE), (forcing_spell, &FORCE)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let candidate = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let hidden = spawn(&mut game, player, source, Zone::Library);
    game.cloak_library_top(player, player);
    assert!(!game.advance_entries());
    let object = game.object_ref(hidden);
    let mut forced = game.clone();

    choose(
        &mut game,
        |action| matches!(action, Action::BeginFaceUp { object: card, .. } if *card == object),
    );
    pay_turn_up(&mut game);
    no_targets(&mut game, object, candidate);

    let spell = spawn(&mut forced, player, forcing_spell, Zone::Hand);
    cast(&mut forced, spell, Some(Target::Card(hidden)));
    act(&mut forced, Action::Pass);
    act(&mut forced, Action::Pass);
    no_targets(&mut forced, object, candidate);
}
