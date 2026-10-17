// Direct typed behavior from the paired complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            min: 2,
            max: 2,
            rule: TargetRule::Grouped {
                base: &TargetRule::Creature,
                constraint: TargetGroupConstraint::DifferentControllers,
            },
        }),
        &[Instruction::ReturnToHand],
    ),
    ..CardRules::EMPTY
};
