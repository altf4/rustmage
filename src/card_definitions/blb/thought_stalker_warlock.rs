pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::MENACE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        program: &[
            Instruction::If {
                guard: StepGuard::OpponentLostLife,
                skip: 2,
            },
            Instruction::RevealHandDiscard { excluded_kinds: 2 },
            Instruction::Skip(1),
            Instruction::Discard(PlayerSet::Targeted, 1),
        ],
    }],
    ..CardRules::EMPTY
};
