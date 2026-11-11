pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::creature_you_control()),
            TargetSpec {
                rule: TargetRule::Grouped {
                    base: &TargetRule::Creature,
                    constraint: TargetGroupConstraint::OtherThanGroup(0),
                },
                min: 2,
                max: 2,
            },
        ]),
        &[Instruction::CreaturePowerDamageBatch {
            source: 0,
            recipients: CreaturePowerDamageRecipients::TargetGroup(1),
        }],
    ),
    ..CardRules::EMPTY
};
