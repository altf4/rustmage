// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::Groups(&[
        TargetSpec::one(TargetRule::Any),
        TargetSpec::one(TargetRule::Grouped { base: &TargetRule::Any, constraint: TargetGroupConstraint::OtherThanGroup(0) }),
    ]), &[Instruction::DamageByGroup { first: 2, other: 1 }]),
    ..CardRules::EMPTY
};
