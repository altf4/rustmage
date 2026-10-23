// Transforming preserves loyalty and this permanent's once-per-turn activation history.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        starting_loyalty: 3,
        transform: Some(TransformSpec {
            front: Card(3000),
            back: Card(5353),
            front_name: "Oko, Lorwyn Liege",
        }),
        ..CardExtras::EMPTY
    },
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
    deck_limit: None,
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
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
                loyalty: Some(-1),
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
                Instruction::SelectMilled(MilledSelectionSpec {
                    filter: MilledFilter::Permanent,
                    count: MilledCount::Range {
                        minimum: 0,
                        maximum: 1,
                    },
                    destination: MilledDestination::Hand,
                }),
            ],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
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
                loyalty: Some(-3),
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5354),
                amount: 2,
                tapped: false,
            }],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
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
                loyalty: Some(-6),
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ChooseCreatureType,
                Instruction::CreateChosenTypeEmblem(Card(5355)),
            ],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
