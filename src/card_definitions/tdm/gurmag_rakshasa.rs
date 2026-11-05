// Source-audited direct typed definition; see the paired provenance record.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::MENACE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Groups(&[
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
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
        ]),
        program: &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: -2,
                toughness: -2,
            },
            Instruction::SelectGroup(1),
            Instruction::Grow {
                subject: Subject::Targets,
                power: 2,
                toughness: 2,
            },
        ],
    }],
    ..CardRules::EMPTY
};
