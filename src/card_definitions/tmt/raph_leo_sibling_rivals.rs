// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::FirstCombatPhase,
            event: &TriggerCondition::SelfAttacks,
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::Attacking,
            },
            min: 1,
            max: 2,
        }),
        program: &[Instruction::Untap, Instruction::AdditionalCombat],
    }],
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [1, 0, 0, 1, 0, 0],
                generic: 1,
            },
            ManaCost {
                colored: [2, 0, 0, 0, 0, 0],
                generic: 1,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
