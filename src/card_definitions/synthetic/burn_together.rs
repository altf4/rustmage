// Adventure face of Callous Sell-Sword, directly authored from Oracle and XMage.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::creature_you_control()),
            TargetSpec::one(TargetRule::Grouped {
                base: &TargetRule::Any,
                constraint: TargetGroupConstraint::OtherThanGroup(0),
            }),
        ]),
        &[
            Instruction::CreaturePowerDamageAny {
                source: 0,
                recipient: 1,
            },
            Instruction::SacrificeTarget { group: 0 },
        ],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(624)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
