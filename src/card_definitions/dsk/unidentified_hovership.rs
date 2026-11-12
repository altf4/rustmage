pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Filtered {
                    base: &TargetRule::Creature,
                    predicate: &TargetPredicate::ToughnessAtMost(5),
                },
                min: 0,
                max: 1,
            }),
            program: &[Instruction::ExileTargetsLinkedToSource],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfLeaves,
            targets: TargetPlan::NONE,
            program: &[Instruction::ManifestDreadForLinkedExileOwners],
        },
    ],
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        guard: ActivationGuard::Always,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::Crew { minimum: 1 }],
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 2,
            toughness: 2,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
