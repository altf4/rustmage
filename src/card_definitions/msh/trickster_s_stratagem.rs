// Gated typed core consumer; whole-card admission requires the complete paid suite.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
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
            TargetSpec {
                rule: TargetRule::creature_you_control(),
                min: 0,
                max: 1,
            },
        ]),
        &[
            Instruction::OwnerLibrarySecondFromTop,
            Instruction::SelectGroup(1),
            Instruction::Connive {
                subject: Subject::Targets,
                amount: 1,
            },
        ],
    ),
    ..CardRules::EMPTY
};
