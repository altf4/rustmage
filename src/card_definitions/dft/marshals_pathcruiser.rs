// Direct typed behavior from complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Search(SearchSpec {
            filter: SearchFilter::BasicLand,
            maximum: 1,
            destination: SearchDestination::Hand,
            tapped: false,
            reveal: true,
        })],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: true,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [1, 1, 1, 1, 1, 0],
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
                    amount: 2,
                },
            ],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost::ZERO,
            targets: TargetPlan::NONE,
            program: &[Instruction::Crew { minimum: 5 }],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NONE,
    extras: &CardExtras {
        exhaust_mask: 1,
        vehicle_stats: Some(Stats {
            power: 6,
            toughness: 5,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
