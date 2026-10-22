// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::MENACE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
            Instruction::If { guard: StepGuard::Expression {
                value: StatValue::Count(CountFilter { zone: CountZone::Graveyard, relation: 0, kind: 0, exclude: 0, subtype: Some("Elf"), basic: false, other: false, cards_only: true }),
                minimum: Some(1), maximum: None,
            }, skip: 2 },
            Instruction::LoseLife(PlayerSet::Opponent, 2),
            Instruction::GainLife(PlayerSet::Controller, 2),
        ],
    }],
    ..CardRules::EMPTY
};
