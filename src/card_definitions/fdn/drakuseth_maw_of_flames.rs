// Direct typed behavior from complete pinned Oracle and XMage implementations.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Any),
            TargetSpec {
                rule: TargetRule::Grouped {
                    base: &TargetRule::Any,
                    constraint: TargetGroupConstraint::OtherThanGroup(0),
                },
                min: 0,
                max: 2,
            },
        ]),
        program: &[Instruction::DamageByGroup { first: 4, other: 3 }],
    }],
    ..CardRules::EMPTY
};
