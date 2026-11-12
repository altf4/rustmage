pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: one(TargetRule::Filtered {
            base: &TargetRule::creature_you_control(),
            predicate: &TargetPredicate::PowerAtMost(2),
        }),
        program: &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 0,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::DEATHTOUCH,
            },
        ],
    }],
    ..CardRules::EMPTY
};
