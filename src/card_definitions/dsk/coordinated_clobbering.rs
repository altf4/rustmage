// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec {
                rule: TargetRule::Filtered {
                    base: &TargetRule::creature_you_control(),
                    predicate: &TargetPredicate::Untapped,
                },
                min: 1,
                max: 2,
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
        &[
            Instruction::SelectGroup(0),
            Instruction::Tap,
            Instruction::GroupPowerDamage {
                source: 0,
                recipient: 1,
            },
        ],
    ),
    ..CardRules::EMPTY
};
