pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::CreatureEnters {
                relation: CastRelation::You,
                other: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Surveil(1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::ClassLevelAtLeast(2),
                event: &TriggerCondition::AttackDeclared {
                    minimum: 1,
                    power_min: None,
                    guard: StepGuard::Always,
                },
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                base: &TargetRule::Filtered {
                    base: &TargetRule::Creature,
                    predicate: &TargetPredicate::Attacking,
                },
                predicate: &TargetPredicate::PowerAtMost(3),
            })),
            program: &[Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::CANT_BE_BLOCKED,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::ClassLevelAtLeast(3),
                event: &TriggerCondition::PlayerDamaged {
                    subject: DamageSubject::Creatures {
                        filter: CreatureFilter {
                            relation: 0,
                            other: false,
                            subtype: None,
                            keyword: Keywords::NONE,
                            attacking: false,
                            tokens: None,
                            artifact: false,
                        },
                        qualification: DamageQualification::Any,
                    },
                    combat_only: true,
                    opponent_only: false,
                    one_or_more: false,
                },
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayIfCombatDamageDealerOnBattlefield { skip: 1 },
                Instruction::BlinkCombatDamageDealer,
            ],
        },
    ],
    activated: &[
        ActivatedAbility {
            guard: ActivationGuard::ClassLevelExactly(1),
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            sorcery: true,
            once_per_turn: false,
            cost: ActivationCost {
                colored: [0, 1, 0, 0, 0, 0],
                generic: 1,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainClassLevel(2)],
        },
        ActivatedAbility {
            guard: ActivationGuard::ClassLevelExactly(2),
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            exhaust: false,
            sorcery: true,
            once_per_turn: false,
            cost: ActivationCost {
                colored: [0, 1, 0, 0, 0, 0],
                generic: 3,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainClassLevel(3)],
        },
    ],
    ..CardRules::EMPTY
};
