// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterUnlessPay {
                cost: ManaCost {
                    colored: [0; 6],
                    generic: 2,
                },
                subject: CounterSubject::Related,
                exile: false,
                unpaid_skip: 0,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacksWhileSaddled,
            targets: TargetPlan::NONE,
            program: &[Instruction::Search(SearchSpec {
                filter: SearchFilter::AllOf(&[
                    SearchFilter::Nonland,
                    SearchFilter::KindsManaValue {
                        kinds: 1 | 16 | 32 | 64 | 256,
                        minimum: 0,
                        maximum: 3,
                    },
                ]),
                maximum: 1,
                destination: SearchDestination::Battlefield,
                tapped: false,
                reveal: false,
            })],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::BecomeSaddled { minimum: 4 }],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
