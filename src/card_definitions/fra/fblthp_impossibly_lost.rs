// Directly authored from the pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::YourTurn,
            event: &TriggerCondition::PlayerDamaged {
                subject: DamageSubject::AnySources,
                combat_only: true,
                opponent_only: true,
                one_or_more: true,
            },
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::WinIfLibraryEmpty,
            Instruction::ShuffleSourceIntoOwnerLibrary,
        ],
    }],
    ..CardRules::EMPTY
};
