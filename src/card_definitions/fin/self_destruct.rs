// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::creature_you_control()),
            TargetSpec::one(TargetRule::Grouped {
                base: &TargetRule::Any,
                constraint: TargetGroupConstraint::OtherThanGroup(0),
            }),
        ]),
        &[Instruction::CreaturePowerDamageBatch {
            source: 0,
            recipients: CreaturePowerDamageRecipients::TargetGroupAndSelf(1),
        }],
    ),
    ..CardRules::EMPTY
};
