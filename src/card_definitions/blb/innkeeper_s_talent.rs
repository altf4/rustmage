// Direct typed behavior from complete pinned Oracle and XMage sources.
const DOUBLE: ReplacementSpec = ReplacementSpec::DoubleCountersPlacedByController {
    condition: StatValue::SourceClassLevelAtLeast(3),
};
const LEVEL: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::ClassLevelExactly(1),
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        colored: [0, 0, 0, 0, 1, 0],
        mana_component: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::GainClassLevel(2)],
    sorcery: true,
    once_per_turn: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::AtStep {
                step: TriggerStep::BeginCombat,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
            program: &[Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::Ward),
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterUnlessPay {
                cost: ManaCost {
                    colored: [0; 6],
                    generic: 1,
                },
                subject: CounterSubject::Related,
                exile: false,
                unpaid_skip: 0,
            }],
        },
    ],
    activated: &[
        LEVEL,
        ActivatedAbility {
            guard: ActivationGuard::ClassLevelExactly(2),
            cost: ActivationCost {
                generic: 3,
                ..LEVEL.cost
            },
            program: &[Instruction::GainClassLevel(3)],
            ..LEVEL
        },
    ],
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::CounterBearingPermanentWard {
                condition: StatValue::SourceClassLevelAtLeast(2),
                ability: 1,
            }],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Replacement(0, DOUBLE)],
        },
    ],
    extras: &CardExtras {
        replacement_effects: &[DOUBLE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
