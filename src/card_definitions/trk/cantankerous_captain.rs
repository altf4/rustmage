// Pinned Oracle and complete XMage class: the total-power check follows the boost.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: one(TargetRule::creature_you_control()),
        program: &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 0,
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::ControlledTotalCreaturePower,
                minimum: 8,
                skip: 1,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::MENACE,
            },
        ],
    }],
    ..CardRules::EMPTY
};
