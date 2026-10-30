// Directly authored from pinned Oracle and XMage; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 0,
            other: true,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[
            Instruction::If {
                guard: StepGuard::ScriedOrSurveilledThisTurn,
                skip: 2,
            },
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::Skip(1),
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
