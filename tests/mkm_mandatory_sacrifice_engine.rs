//! Mandatory resolution sacrifice checks use legal payment and preserve actual events.
mod support;

use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const ARTIFACT: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 16,
    nonland: false,
    other: false,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};
const COUNTERS: Instruction = Instruction::PutCounters {
    subject: Subject::Source,
    kind: CounterKind::PlusOne,
    amount: 2,
};

const fn required_program(amount: u32) -> [Instruction; 4] {
    [
        Instruction::RememberCount(AmountValue::Fixed(9)),
        Instruction::ChooseObjects {
            kind: EffectChoiceKind::SacrificeAsCost,
            players: PlayerSet::Controller,
            filter: ARTIFACT,
            amount,
            optional_skip: None,
        },
        Instruction::IfAmountAtLeast {
            value: AmountValue::MovedCount,
            minimum: 1,
            skip: 1,
        },
        COUNTERS,
    ]
}

const REQUIRED: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &required_program(1),
    }],
    ..CardRules::EMPTY
};
const REQUIRED_TWO: CardRules = CardRules {
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &required_program(2),
    }],
    ..REQUIRED
};
const PROHIBITION: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledPermanentsCannotBeSacrificed],
    }],
    ..CardRules::EMPTY
};
const OPTIONAL: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Sacrifice,
                players: PlayerSet::Controller,
                filter: ARTIFACT,
                amount: 1,
                optional_skip: Some(1),
            },
            COUNTERS,
        ],
    }],
    ..CardRules::EMPTY
};

fn start(game: &mut Game, player: Player) -> CardId {
    let source = put(game, player, "Hill Giant", Zone::Hand);
    cast(game, source, None);
    for _ in 0..4 {
        act(game, Action::Pass);
    }
    source
}

#[test]
fn replacement_modified_sacrifices_pay_the_required_cost() {
    let card = Card::from_name("Hill Giant").unwrap();
    let _rules = fixture_rules::activate(&[(card, &REQUIRED)]);
    struct ResetPrevention;
    impl Drop for ResetPrevention {
        fn drop(&mut self) {
            zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(None));
        }
    }
    let _prevention = ResetPrevention;
    let player = Player::P0;
    let mut root = scenario(player);
    let nonartifact = put(&mut root, player, "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut root, player, "Candy Trail", Zone::Battlefield);
    let exact = root.object_ref(artifact);
    let sacrifice = Action::Sacrifice { object: exact };

    let mut redirected = root.clone();
    put(
        &mut redirected,
        player.other(),
        "Rest in Peace",
        Zone::Battlefield,
    );
    let source = start(&mut redirected, player);
    assert_eq!(redirected.state.pending, Pending::EffectChoice { player });
    reject(&mut redirected, Action::DeclineEffectPayment);
    let other = redirected.object_ref(nonartifact);
    reject(&mut redirected, Action::Sacrifice { object: other });
    act(&mut redirected, sacrifice.clone());
    assert_eq!(redirected.object_view(artifact).zone, Zone::Exile);
    assert_eq!(redirected.counter_count(source, CounterKind::PlusOne), 2);
    assert_eq!(
        redirected.state.players[player.index()].permanents_sacrificed_this_turn,
        1
    );

    let source = start(&mut root, player);
    assert_eq!(root.state.pending, Pending::EffectChoice { player });
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(exact)));
    let saved = root.snapshot();
    let revision = root.revision();
    let index = root
        .legal_actions()
        .iter()
        .position(|action| *action == sacrifice)
        .unwrap();
    let mut indexed = root.clone();
    act(&mut root, sacrifice);
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(root.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(indexed.snapshot(), saved);
    assert_eq!(root.object_ref(artifact), exact);
    assert!(root.on_battlefield(artifact));
    assert_eq!(root.counter_count(source, CounterKind::PlusOne), 2);
    assert_eq!(
        root.state.players[player.index()].permanents_sacrificed_this_turn,
        0
    );
}

#[test]
fn required_sacrifice_without_a_legal_payment_clears_the_previous_count() {
    let card = Card::from_name("Hill Giant").unwrap();
    let prohibition = Card::from_name("Short Bow").unwrap();
    let _rules = fixture_rules::activate(&[(card, &REQUIRED), (prohibition, &PROHIBITION)]);
    let player = Player::P0;
    let mut root = scenario(player);
    put(&mut root, player, "Bear Cub", Zone::Battlefield);

    let mut empty = root.clone();
    let source = start(&mut empty, player);
    assert_eq!(empty.state.pending, Pending::Priority);
    assert_eq!(empty.counter_count(source, CounterKind::PlusOne), 0);

    let artifact = put(&mut root, player, "Candy Trail", Zone::Battlefield);
    put(&mut root, player, "Short Bow", Zone::Battlefield);
    let source = start(&mut root, player);
    assert_eq!(root.state.pending, Pending::Priority);
    assert!(root.on_battlefield(artifact));
    assert_eq!(root.counter_count(source, CounterKind::PlusOne), 0);
}

#[test]
fn a_fixed_sacrifice_cost_requires_the_entire_payment() {
    let card = Card::from_name("Hill Giant").unwrap();
    let _rules = fixture_rules::activate(&[(card, &REQUIRED_TWO)]);
    let player = Player::P0;
    let mut game = scenario(player);
    let first = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let mut short = game.clone();
    let source = start(&mut short, player);
    assert_eq!(short.state.pending, Pending::Priority);
    assert!(short.on_battlefield(first));
    assert_eq!(short.counter_count(source, CounterKind::PlusOne), 0);

    let second = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let source = start(&mut game, player);
    let first_sacrifice = Action::Sacrifice {
        object: game.object_ref(first),
    };
    let second_sacrifice = Action::Sacrifice {
        object: game.object_ref(second),
    };
    act(&mut game, first_sacrifice.clone());
    assert_eq!(game.state.pending, Pending::EffectChoice { player });
    assert!(game.on_battlefield(first));
    reject(&mut game, first_sacrifice);
    act(&mut game, second_sacrifice);
    assert_eq!(game.object_view(first).zone, Zone::Graveyard);
    assert_eq!(game.object_view(second).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 2);
    assert_eq!(
        game.state.players[player.index()].permanents_sacrificed_this_turn,
        2
    );
}

#[test]
fn the_ordinary_optional_sacrifice_still_offers_decline() {
    let card = Card::from_name("Hill Giant").unwrap();
    let _rules = fixture_rules::activate(&[(card, &OPTIONAL)]);
    let player = Player::P1;
    let mut game = scenario(player);
    let artifact = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let source = start(&mut game, player);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    assert!(declined.on_battlefield(artifact));
    assert_eq!(declined.counter_count(source, CounterKind::PlusOne), 0);

    let exact = game.object_ref(artifact);
    act(&mut game, Action::Sacrifice { object: exact });
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 2);
}
