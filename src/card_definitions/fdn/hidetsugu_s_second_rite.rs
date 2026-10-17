// Direct typed behavior from the complete paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::Player { opponent_only: false }),
        &[Instruction::IfTargetPlayerLife { life: 10, skip: 1 }, Instruction::Damage(10)]),
    ..CardRules::EMPTY
};
