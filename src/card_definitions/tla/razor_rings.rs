// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Filtered {
            base: &TargetRule::Creature,
            predicate: &TargetPredicate::AttackingOrBlocking,
        }),
        &[
            Instruction::DamageForExcess(AmountValue::Fixed(4)),
            Instruction::GainExcessDamageLife,
        ],
    ),
    ..CardRules::EMPTY
};
