// Direct typed behavior from the paired complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[
            Instruction::If {
                guard: StepGuard::CreatureDied,
                skip: 2,
            },
            Instruction::Grow {
                subject: Subject::Targets,
                power: -13,
                toughness: -13,
            },
            Instruction::Skip(1),
            Instruction::Grow {
                subject: Subject::Targets,
                power: -1,
                toughness: -1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
