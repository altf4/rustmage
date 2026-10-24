// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::DamageScaled(AmountValue::ManaSpent),
            Instruction::UntapCreatures(CreatureFilter {
                relation: 0,
                other: false,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            }),
        ],
    ),
    extras: &CardExtras {
        x_symbols: 1,
        flashback_cost: Some(ManaCost {
            colored: [1, 0, 0, 1, 0, 0],
            generic: 6,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
