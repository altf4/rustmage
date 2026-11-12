// Direct typed behavior from complete pinned Oracle and XMage sources.
const MICE: StatValue = StatValue::Count(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Mouse"),
    basic: false,
    other: false,
    cards_only: false,
});
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::TRAMPLE,
            },
            Instruction::If {
                guard: StepGuard::Expression {
                    value: MICE,
                    minimum: Some(1),
                    maximum: None,
                },
                skip: 1,
            },
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 0,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};
