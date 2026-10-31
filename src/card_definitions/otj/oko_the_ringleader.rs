// Pinned Oracle and XMage source recorded beside this file. Gated until the
// crime commit boundaries and multi-copy interactions have been validated.
const fn loyalty(amount: i32) -> ActivationCost {
    ActivationCost {
        loyalty: Some(amount),
        ..ActivationCost::ZERO
    }
}
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Unsupported,
    extras: &CardExtras {
        starting_loyalty: 3,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::creature_you_control(),
            min: 0,
            max: 1,
        }),
        program: &[Instruction::BecomeCopy(InPlaceCopySpec {
            recipient_group: None,
            blueprint_group: 0,
            overrides: crate::engine::CopyOverrides {
                name: None,
                legendary: None,
                kinds: None,
                types: None,
                keywords: Keywords::HEXPROOF,
            },
            add_kinds: 0,
            base: None,
            next_turn: false,
        })],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(1),
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 2),
                Instruction::If {
                    guard: StepGuard::CommittedCrimeThisTurn,
                    skip: 2,
                },
                Instruction::Discard(PlayerSet::Controller, 1),
                Instruction::Skip(1),
                Instruction::Discard(PlayerSet::Controller, 2),
            ],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: loyalty(-1),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5474),
                amount: 1,
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
            cost: loyalty(-5),
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyOtherControlledNonlands],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
