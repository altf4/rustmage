pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayMana {
                    cost: ManaCost {
                        colored: [0, 1, 0, 0, 0, 0],
                        generic: 0,
                    },
                    skip: 1,
                },
                Instruction::Reflexive { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Filtered {
                    base: &TargetRule::Creature,
                    predicate: &TargetPredicate::Attacking,
                },
                predicate: &TargetPredicate::OtherThanSource,
            }),
            program: &[Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::CANT_BE_BLOCKED,
            }],
        },
    ],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::Crew { minimum: 2 }],
    }],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 4,
            toughness: 3,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
