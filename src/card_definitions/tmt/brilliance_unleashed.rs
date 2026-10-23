// Complete direct typed Oracle/XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Deal 5 damage to target creature.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::Damage(5)],
        },
        SpellMode {
            text: "Return target artifact card from your graveyard; a noncreature card returns as a 3/3 Robot with flying.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 16,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::ReturnArtifactAsRobot],
        },
        SpellMode {
            text: "Choose both",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
                TargetSpec::one(TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 16,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
            ]),
            program: &[
                Instruction::SelectGroup(0),
                Instruction::Damage(5),
                Instruction::SelectGroup(1),
                Instruction::ReturnArtifactAsRobot,
            ],
        },
    ]),
    ..CardRules::EMPTY
};
