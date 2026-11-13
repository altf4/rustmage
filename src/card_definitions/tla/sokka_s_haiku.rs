// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Spell),
            TargetSpec::one(object(2)),
        ]),
        &[
            Instruction::Counter,
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
            Instruction::SelectGroup(1),
            Instruction::Untap,
        ],
    ),
    ..CardRules::EMPTY
};
