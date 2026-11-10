pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Target artifact or creature gains indestructible until end of turn.",
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1 | 16,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::INDESTRUCTIBLE,
            }],
        },
        SpellMode {
            text: "Reroute Systems deals 2 damage to target tapped creature.",
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: -1,
                other: false,
                nonland: false,
                tapped: true,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::Damage(2)],
        },
    ]),
    ..CardRules::EMPTY
};
