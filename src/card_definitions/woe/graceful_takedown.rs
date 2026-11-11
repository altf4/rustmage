// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec {
                rule: TargetRule::Filtered {
                    base: &TargetRule::creature_you_control(),
                    predicate: &TargetPredicate::Enchanted,
                },
                min: 0,
                max: usize::MAX,
            },
            TargetSpec {
                rule: TargetRule::Grouped {
                    base: &TargetRule::creature_you_control(),
                    constraint: TargetGroupConstraint::OtherThanGroup(0),
                },
                min: 0,
                max: 1,
            },
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
        ]),
        &[Instruction::GroupsPowerDamage {
            sources: &[0, 1],
            recipient: 2,
        }],
    ),
    ..CardRules::EMPTY
};
