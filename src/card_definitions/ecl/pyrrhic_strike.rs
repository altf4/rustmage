// Direct typed behavior from the pinned Oracle and XMage sources.
const ARTIFACT_OR_ENCHANTMENT: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 48,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const CREATURE_AT_LEAST_THREE: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 3,
    max_mana: u32::MAX,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Destroy target artifact or enchantment.",
            targets: TargetPlan::Single(TargetSpec::one(ARTIFACT_OR_ENCHANTMENT)),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Destroy target creature with mana value 3 or greater.",
            targets: TargetPlan::Single(TargetSpec::one(CREATURE_AT_LEAST_THREE)),
            program: &[Instruction::Destroy],
        },
        SpellMode {
            text: "Both, in printed order.",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(ARTIFACT_OR_ENCHANTMENT),
                TargetSpec::one(CREATURE_AT_LEAST_THREE),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::Destroy,
                Instruction::SelectGroup(1),
                Instruction::Destroy,
            ],
        },
    ]),
    extras: &CardExtras {
        blight: Some(BlightSpec {
            amount: 2,
            normal_modes: 3,
            paid_modes: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
