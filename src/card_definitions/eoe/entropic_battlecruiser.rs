const BASE: BaseStatSetter = BaseStatSetter {
    power: Some(3),
    toughness: Some(10),
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
                keywords: Keywords::FLYING.union(Keywords::DEATHTOUCH),
            },
            StaticAbilityFragment::BaseStatSetter(0, BASE),
        ],
    }],
    extras: &CardExtras {
        base_stat_setter: Some(BASE),
        ..CardExtras::EMPTY
    },
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::Expression {
                    value: StatValue::SourceCounters(CounterKind::Charge),
                    minimum: Some(1),
                    maximum: None,
                },
                event: &TriggerCondition::Discarded {
                    relation: CastRelation::Opponent,
                    one_or_more: false,
                },
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::LoseLife(PlayerSet::Opponent, 3)],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::Expression {
                    value: StatValue::SourceCounters(CounterKind::Charge),
                    minimum: Some(8),
                    maximum: None,
                },
                event: &TriggerCondition::SelfAttacks,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::DiscardRememberCount {
                    players: PlayerSet::Opponent,
                    amount: 1,
                },
                Instruction::IfAmountAtLeast {
                    value: AmountValue::MovedCount,
                    minimum: 1,
                    skip: 1,
                },
                Instruction::Skip(1),
                Instruction::LoseLife(PlayerSet::Opponent, 3),
            ],
        },
    ],
    activated: &[ActivatedAbility {
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
    }],
    ..CardRules::EMPTY
};
