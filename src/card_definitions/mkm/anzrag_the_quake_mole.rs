pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(8, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfBecomesBlocked,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::UntapPlayerCreatures(PlayerSet::Controller),
            Instruction::AdditionalCombat,
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 0, 2, 2, 0],
            generic: 3,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::RequireSubjectBlocked {
            subject: Subject::Source,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
