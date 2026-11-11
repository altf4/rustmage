const NONCREATURE: TriggerCondition = TriggerCondition::SpellCast {
    relation: CastRelation::You,
    filter: CastFilter::Noncreature,
    ordinal: None,
};
const PROWESS: &[Instruction] = &[Instruction::Grow {
    subject: Subject::Source,
    power: 1,
    toughness: 1,
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: NONCREATURE,
            targets: TargetPlan::NONE,
            program: PROWESS,
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&NONCREATURE),
            targets: TargetPlan::NONE,
            program: PROWESS,
        },
        TriggeredAbility {
            condition: NONCREATURE,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
            program: &[Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::CANT_BE_BLOCKED,
            }],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledCreatureTrigger {
            ability: 1,
            other: true,
            nontoken: false,
        }],
    }],
    ..CardRules::EMPTY
};
