// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(object(1 | 16)),
        &[
            Instruction::GainControlUntilEndTurn,
            Instruction::Untap,
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::HASTE,
            },
            Instruction::IfSubjectSubtype {
                subject: Subject::Targets,
                subtype: "Spacecraft",
                skip: 1,
            },
            Instruction::PutCountersThenScheduleEndStep {
                subject: Subject::Targets,
                kind: CounterKind::Charge,
                amount: 10,
                ability: 0,
            },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::RemoveDelayedObjectCounters {
            kind: CounterKind::Charge,
            amount: 10,
        }],
    }],
    ..CardRules::EMPTY
};
