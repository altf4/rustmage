// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GrowScaled {
                subject: Subject::Source,
                amount: AmountValue::Stat(StatValue::AllAttackingCreatures),
                power: 1,
                toughness: 0,
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::AllAttackingSubtype("Rat")),
                minimum: 1,
                skip: 1,
            },
            Instruction::GrantKeywords {
                subject: Subject::Source,
                keywords: Keywords::TRAMPLE,
            },
        ],
    }],
    ..CardRules::EMPTY
};
