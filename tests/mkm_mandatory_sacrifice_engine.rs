//! A mandatory sacrifice remembers only the actual result of its own batch.
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

const REQUIRED: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::RememberCount(AmountValue::Fixed(9)),
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::SacrificeRememberCount,
                players: PlayerSet::Controller,
                filter: ARTIFACT,
                amount: 1,
                optional_skip: None,
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::MovedCount,
                minimum: 1,
                skip: 1,
            },
            COUNTERS,
        ],
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
fn a_mandatory_choice_uses_fresh_actual_sacrifices_even_when_the_destination_changes() {
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

    let mut empty = root.clone();
    let source = start(&mut empty, player);
    assert_eq!(empty.state.pending, Pending::Priority);
    assert_eq!(empty.counter_count(source, CounterKind::PlusOne), 0);

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
    reject(&mut redirected, Action::ChooseOptional { accept: false });
    let other = redirected.object_ref(nonartifact);
    reject(&mut redirected, Action::Sacrifice { object: other });
    act(&mut redirected, sacrifice.clone());
    assert_eq!(redirected.object_view(artifact).zone, Zone::Exile);
    assert_eq!(redirected.counter_count(source, CounterKind::PlusOne), 2);

    let source = start(&mut root, player);
    assert_eq!(root.state.pending, Pending::EffectChoice { player });
    zone_replacements::TEST_PREVENT_ZONE_MOVE.with(|rule| rule.set(Some(exact)));
    act(&mut root, sacrifice);
    assert_eq!(root.object_ref(artifact), exact);
    assert!(root.on_battlefield(artifact));
    assert_eq!(root.counter_count(source, CounterKind::PlusOne), 0);
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
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert!(declined.on_battlefield(artifact));
    assert_eq!(declined.counter_count(source, CounterKind::PlusOne), 0);

    let exact = game.object_ref(artifact);
    act(&mut game, Action::Sacrifice { object: exact });
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 2);
}
