pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::Damage(6),
            Instruction::MayDestroyAttachedEquipment { recipient: 0 },
        ],
    ),
    ..CardRules::EMPTY
};
