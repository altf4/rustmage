const BASE: BaseStatSetter = BaseStatSetter {
    power: Some(1),
    toughness: Some(5),
    own_turn: false,
    condition: Some(StatValue::AtLeast {
        value: &StatValue::SourceCounters(CounterKind::Charge),
        minimum: 8,
    }),
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::SourceCounterCreature {
                own_turn: false,
                kind: CounterKind::Charge,
                minimum: 8,
                form: SourceCreatureForm::AddCreature,
                subtypes: &[],
                keywords: Keywords::FLYING,
            },
            StaticAbilityFragment::BaseStatSetter(0, BASE),
        ],
    }],
    extras: &CardExtras {
        base_stat_setter: Some(BASE),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
        program: &[Instruction::Damage(3)],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::TapPermanents(1),
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 1,
                        nonland: false,
                        other: true,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: None,
                    },
                }),
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutCountersScaled {
                subject: Subject::Source,
                kind: CounterKind::Charge,
                amount: AmountValue::TappedCostCreaturePower,
            }],
            sorcery: true,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::SourceCounterAtLeast {
                kind: CounterKind::Charge,
                minimum: 8,
            },
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 1,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Grow {
                subject: Subject::Source,
                power: 2,
                toughness: 0,
            }],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
