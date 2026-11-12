const RETURN: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Filtered {
        base: &TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 0,
            relation: 0,
            other: false,
            nonland: true,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        },
        predicate: &TargetPredicate::Permanent,
    },
    predicate: &TargetPredicate::WithoutKinds(1),
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(5622),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::ClassLevelAtLeast(2),
                event: &TriggerCondition::FilteredEntersBatchYou {
                    predicate: EntryPredicate::NoncreatureNonland,
                    other: false,
                },
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
            program: &[Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::BecomesClassLevel(3),
            targets: TargetPlan::Single(TargetSpec::one(RETURN)),
            program: &[Instruction::ReturnToBattlefield],
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
                colored: [1, 0, 0, 0, 0, 0],
                generic: 0,
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
                colored: [1, 0, 0, 0, 0, 0],
                generic: 4,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GainClassLevel(3)],
        },
    ],
    ..CardRules::EMPTY
};
