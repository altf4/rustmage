// Pinned Oracle and complete XMage class: hand discard cost and targeted end step.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::MENACE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: one(TargetRule::Player {
            opponent_only: true,
        }),
        program: &[Instruction::Discard(PlayerSet::Targeted, 2)],
    }],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Hand,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 2,
            mana_component: true,
            discard: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Creature),
        program: &[Instruction::Grow {
            subject: Subject::Targets,
            power: -4,
            toughness: -4,
        }],
    }],
    ..CardRules::EMPTY
};
