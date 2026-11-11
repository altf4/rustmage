pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Filtered {
            base: &TargetRule::Creature,
            predicate: &TargetPredicate::AttackingOrBlocking,
        }),
        &[
            Instruction::IfCastWithControlledSubtype { skip: 2 },
            Instruction::Damage(4),
            Instruction::Skip(1),
            Instruction::Damage(2),
        ],
    ),
    extras: &CardExtras {
        cast_controlled_subtype: Some("Mount"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
