// Direct typed behavior from complete pinned Oracle/XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(2)),
                Instruction::Draw(PlayerSet::Controller, 2),
                Instruction::Discard(PlayerSet::Controller, 2),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::ClassLevel { level: 2 },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 16,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 2,
            }),
            program: &[Instruction::ReturnToHand],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::ClassLevelAtLeast(3),
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::BeginCombat,
                    relation: CastRelation::You,
                    guard: StepGuard::Always,
                },
            },
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 16,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Targets,
                    kind: CounterKind::PlusOne,
                    amount: 3,
                },
                Instruction::AnimateTargetsIfNoncreature(AnimationSpec {
                    artifact: false,
                    creature_types: AnimationCreatureTypes::AddNamed(&["Robot"]),
                    base: Some((0, 0)),
                    keywords: Keywords::NONE,
                    duration: AnimationDuration::Permanent,
                }),
            ],
        },
    ],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [0, 1, 0, 0, 0, 0],
                generic: 1,
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
                colored: [0, 1, 0, 0, 0, 0],
                generic: 4,
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
