// Direct typed behavior from the pinned Oracle and XMage sources.
const FOOD_COST: ActivationCost = ActivationCost {
    generic: 2,
    mana_component: true,
    tap: true,
    sacrifice: true,
    ..ActivationCost::ZERO
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[
            Instruction::GainControlUntilEndTurn,
            Instruction::Untap,
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::HASTE,
            },
        ],
    }],
    activated: &[
        ActivatedAbility {
            cost: FOOD_COST,
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            cost: FOOD_COST,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
                opponent_only: true,
            })),
            program: &[Instruction::LoseLife(PlayerSet::Targeted, 3)],
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            sorcery: false,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
