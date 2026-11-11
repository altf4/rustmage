pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::VIGILANCE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::VIGILANCE,
        )],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 2,
            }),
            program: &[Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::ControlledCreatureAttacksWithCounter(CounterKind::PlusOne),
            targets: TargetPlan::NONE,
            program: &[Instruction::DoubleCounterKind {
                subject: Subject::Entering,
                kind: CounterKind::PlusOne,
            }],
        },
    ],
    ..CardRules::EMPTY
};
