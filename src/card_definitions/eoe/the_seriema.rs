const CHARGED: StatValue = StatValue::AtLeast {
    value: &StatValue::SourceCounters(CounterKind::Charge),
    minimum: 7,
};
const BASE: BaseStatSetter = BaseStatSetter {
    power: Some(5),
    toughness: Some(5),
    own_turn: false,
    condition: Some(CHARGED),
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::SourceCounterCreature {
                own_turn: false,
                kind: CounterKind::Charge,
                minimum: 7,
                form: SourceCreatureForm::ArtifactCreature,
                subtypes: &[],
                keywords: Keywords::FLYING,
            },
            StaticAbilityFragment::BaseStatSetter(0, BASE),
        ],
    },
    StaticAbilityGroup {
        line: 3,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::ControlledOtherTappedLegendaryCreatureKeywords {
                condition: CHARGED,
                keywords: Keywords::INDESTRUCTIBLE,
            },
        ],
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GROUPS,
    extras: &CardExtras {
        base_stat_setter: Some(BASE),
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Search(SearchSpec {
            filter: SearchFilter::AllOf(&[SearchFilter::Legendary, SearchFilter::Creature]),
            maximum: 1,
            destination: SearchDestination::Hand,
            tapped: false,
            reveal: true,
        })],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
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
