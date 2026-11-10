const STATIONED: StatValue = StatValue::AtLeast {
    value: &StatValue::SourceCounters(CounterKind::Charge),
    minimum: 12,
};
const BASE: BaseStatSetter = BaseStatSetter {
    power: Some(7),
    toughness: Some(15),
    own_turn: false,
    condition: Some(STATIONED),
};
const MULTICOLORED: AmountValue = AmountValue::Stat(StatValue::MulticoloredPermanentsYouControl);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::SourceCounterCreature {
                own_turn: false,
                kind: CounterKind::Charge,
                minimum: 12,
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
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokensScaled {
                token: Card(5309),
                amount: MULTICOLORED,
                tapped: true,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::Expression {
                    value: StatValue::SourceCounters(CounterKind::Charge),
                    minimum: Some(12),
                    maximum: None,
                },
                event: &TriggerCondition::SelfAttacks,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::DrawScaled(PlayerSet::Controller, MULTICOLORED)],
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
                    kinds: 1,
                    other: true,
                    keywords: Keywords::NONE,
                    source_only: false,
                    nonland: false,
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
