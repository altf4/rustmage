// Direct typed behavior from complete pinned Oracle and XMage sources.
const OTHER_LIZARDS: StatValue = StatValue::Count(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Lizard"),
    basic: false,
    other: true,
    cards_only: false,
});
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 2),
    keywords: Keywords::TRAMPLE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[
            Instruction::IfAmountAtLeast {
                value: AmountValue::TargetCreaturePower,
                minimum: 3,
                skip: 1,
            },
            Instruction::If {
                guard: StepGuard::Expression {
                    value: OTHER_LIZARDS,
                    minimum: Some(1),
                    maximum: None,
                },
                skip: 3,
            },
            Instruction::GainControlUntilEndTurn,
            Instruction::Untap,
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::HASTE,
            },
        ],
    }],
    ..CardRules::EMPTY
};
