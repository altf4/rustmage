// Direct typed behavior from complete pinned Oracle/XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::AttackDeclared {
                minimum: 1,
                power_min: None,
                guard: StepGuard::Always,
            },
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::Attacking,
            }),
            program: &[Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::ClassLevelAtLeast(2),
                event: &TriggerCondition::ControlledCreatureLeaves { with_counter: true },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainLife(PlayerSet::Controller, 2)],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::ClassLevelAtLeast(3),
                event: &TriggerCondition::SpellCast {
                    relation: CastRelation::You,
                    filter: CastFilter::Any,
                    ordinal: None,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCountersMatching {
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
    ],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 2,
                alternatives: &[],
                additional: None,
                mana_component: true,
                tap: false,
                sacrifice: false,
                exile_self: false,
                discard: false,
                life: 0,
                remove: None,
                loyalty: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::SetClassLevel(2)],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 3,
                alternatives: &[],
                additional: None,
                mana_component: true,
                tap: false,
                sacrifice: false,
                exile_self: false,
                discard: false,
                life: 0,
                remove: None,
                loyalty: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::SetClassLevel(3)],
            sorcery: true,
            once_per_turn: false,
        },
    ],
    extras: &CardExtras {
        class_level_activations: &[(0, 2), (1, 3)],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
