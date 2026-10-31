// Partial back face. The chosen-type emblem remains gated.
const fn loyalty(n: i32) -> ActivationCost {
    ActivationCost {
        colored: [0; 6],
        generic: 0,
        alternatives: &[],
        additional: None,
        mana_component: false,
        tap: false,
        sacrifice: false,
        exile_self: false,
        discard: false,
        life: 0,
        remove: None,
        loyalty: Some(n),
    }
}
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Unsupported,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Main1,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [0, 1, 0, 0, 0, 0],
                    generic: 0,
                },
                skip: 1,
            },
            Instruction::TransformSource,
        ],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(-1),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
                Instruction::RecoverMilled(RecoveryFilter::Permanent),
            ],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(-3),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5471),
                amount: 2,
                tapped: false,
            }],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(-6),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateEmblemForChosenCreatureType(Card(5472))],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    extras: &CardExtras {
        starting_loyalty: 3,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
