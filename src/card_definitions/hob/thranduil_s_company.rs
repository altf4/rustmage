pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ConditionalAdditionalLandPlays {
            condition: StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: Some("Elf"),
                basic: false,
                other: true,
                cards_only: false,
            }),
            amount: 1,
        }],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LandEntersYou,
        targets: one(TargetRule::creature_you_control()),
        program: &[
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::VIGILANCE,
            },
        ],
    }],
    ..CardRules::EMPTY
};
