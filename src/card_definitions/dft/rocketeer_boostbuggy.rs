// Exhaust animation is permanent; the independent Crew animation lasts until cleanup.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5302),
            amount: 1,
            tapped: false,
        }],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: true,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                generic: 3,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::AnimateSource(AnimationSpec {
                    artifact: true,
                    creature_types: AnimationCreatureTypes::Unchanged,
                    base: None,
                    keywords: Keywords::NONE,
                    duration: AnimationDuration::Permanent,
                }),
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
            ],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost::ZERO,
            targets: TargetPlan::NONE,
            program: &[Instruction::Crew { minimum: 1 }],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    extras: &CardExtras {
        exhaust_mask: 1,
        vehicle_stats: Some(Stats {
            power: 3,
            toughness: 2,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
