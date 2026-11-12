// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card::from_catalogue_id(5296),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::InterveningIf {
                event: &TriggerCondition::AtStep {
                    step: TriggerStep::End,
                    relation: CastRelation::You,
                    guard: StepGuard::Always,
                },
                guard: StepGuard::SourceUnsolved(&StepGuard::Expression {
                    value: StatValue::Count(CountFilter {
                        zone: CountZone::Battlefield,
                        relation: 0,
                        kind: 16,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: false,
                    }),
                    minimum: Some(3),
                    maximum: None,
                }),
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::SolveSource],
        },
    ],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::SourceSolved,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 2,
            mana_component: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 16,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::WithoutKinds(1),
        }),
        program: &[
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 4,
            },
            Instruction::AnimateTargets(AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Additional(&["Bird"]),
                base: Some((0, 0)),
                keywords: Keywords::FLYING,
                duration: AnimationDuration::Permanent,
            }),
        ],
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
