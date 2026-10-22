// Direct typed loyalty behavior; effects survive source departure.
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
                    colored: [0, 0, 0, 0, 1, 0],
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
                loyalty: Some(2),
            },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 1,
            }),
            program: &[Instruction::GainAllCreatureTypesPermanently {
                subject: Subject::Targets,
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
                loyalty: Some(1),
            },
            targets: one(TargetRule::Creature),
            program: &[Instruction::GrowUntilNextTurn {
                subject: Subject::Targets,
                stats: Stats {
                    power: -2,
                    toughness: 0,
                },
            }],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
