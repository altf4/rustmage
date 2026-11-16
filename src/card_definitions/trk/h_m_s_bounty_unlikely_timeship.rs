// Oracle and official Star Trek release notes; pinned XMage class is absent.
const BASE: BaseStatSetter = BaseStatSetter {
    power: Some(3),
    toughness: Some(4),
    own_turn: false,
    condition: Some(StatValue::AtLeast {
        value: &StatValue::SourceCounters(CounterKind::Charge),
        minimum: 8,
    }),
};
const WHALE_PERMANENTS: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Whale"),
    basic: false,
    other: false,
    cards_only: false,
};
const WHALE_COUNT: AmountValue = AmountValue::Sum(&[
    AmountValue::Stat(StatValue::TappedCount(WHALE_PERMANENTS)),
    AmountValue::Stat(StatValue::UntappedCount(WHALE_PERMANENTS)),
]);
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 1,
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
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::Search(SearchSpec {
                filter: SearchFilter::Subtype("Whale"),
                maximum: 1,
                destination: SearchDestination::Hand,
                tapped: false,
                reveal: true,
            })],
        },
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Source,
                combat_only: true,
                opponent_only: false,
                one_or_more: false,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::IfAmountAtLeast {
                    value: WHALE_COUNT,
                    minimum: 1,
                    skip: 2,
                },
                Instruction::If {
                    guard: StepGuard::NoExtraTurnBegunThisGame,
                    skip: 1,
                },
                Instruction::ExtraTurn {
                    power_up_prohibited: false,
                },
            ],
        },
    ],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        sorcery: true,
        once_per_turn: false,
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
    }],
    ..CardRules::EMPTY
};
