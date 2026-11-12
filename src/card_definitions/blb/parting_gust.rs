const TARGET: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
    base: &TargetRule::Creature,
    predicate: &TargetPredicate::Nontoken,
}));

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TARGET,
        &[
            Instruction::GiveGift,
            Instruction::IfGift {
                promised: true,
                skip: 1,
            },
            Instruction::Exile,
            Instruction::IfGift {
                promised: false,
                skip: 1,
            },
            Instruction::BlinkUntilEndStepWithCounter {
                counter: CounterKind::PlusOne,
                actual_exile_only: false,
            },
        ],
    ),
    extras: &CardExtras {
        gift: Some(GiftSpec {
            kind: GiftKind::TappedFish,
            token: Some(Card(5317)),
            promised_targets: TARGET,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
