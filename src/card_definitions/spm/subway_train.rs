// Direct typed rules from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [0, 0, 0, 0, 1, 0],
                    generic: 0,
                },
                skip: 1,
            },
            Instruction::Search(SearchSpec {
                filter: SearchFilter::BasicLand,
                maximum: 1,
                destination: SearchDestination::Hand,
                tapped: false,
                reveal: true,
            }),
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::Crew { minimum: 2 }],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 3,
            toughness: 1,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
