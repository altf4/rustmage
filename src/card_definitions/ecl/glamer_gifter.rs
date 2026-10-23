// Direct typed behavior; complete Oracle body and pinned XMage sources reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    keywords: Keywords::FLASH.union(Keywords::FLYING),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: -1,
                other: true,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 1,
        }),
        program: &[
            Instruction::SetBaseStats {
                subject: Subject::Targets,
                power: Some(4),
                toughness: Some(4),
            },
            Instruction::GainAllCreatureTypesUntilEndTurn {
                subject: Subject::Targets,
            },
        ],
    }],
    ..CardRules::EMPTY
};
