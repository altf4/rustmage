// Complete typed behavior from full pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 7),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersOrAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileOpponentUntilNonlandLinked],
        },
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::End,
                relation: CastRelation::You,
                guard: StepGuard::PermanentLeftBattlefieldThisTurn,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::CastOneLinkedDuringResolution],
        },
    ],
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [0, 1, 1, 0, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 2, 0, 0, 0, 0],
                generic: 4,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
