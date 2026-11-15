// Complete Oracle station and cloaking behavior, checked against pinned XMage.
const STATIONED: StatValue = StatValue::AtLeast {
    value: &StatValue::SourceCounters(CounterKind::Charge),
    minimum: 8,
};
const BASE: BaseStatSetter = BaseStatSetter {
    power: Some(4),
    toughness: Some(4),
    own_turn: false,
    condition: Some(STATIONED),
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::SourceConditionalKeywords {
                condition: StatValue::SourceNoncreature,
                keywords: Keywords::HEXPROOF,
            }],
        },
        StaticAbilityGroup {
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
                StaticAbilityFragment::BaseStatSetter(1, BASE),
            ],
        },
    ],
    extras: &CardExtras {
        base_stat_setter: Some(BASE),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            event: &TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Noncreature,
                ordinal: None,
            },
            guard: StepGuard::SourceNoncreature,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::Charge,
            amount: 2,
        }],
    }],
    activated: &[ActivatedAbility {
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
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        sorcery: true,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    ..CardRules::EMPTY
};
