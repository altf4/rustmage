// Complete Oracle behavior, checked against pinned XMage or official release notes.
const BASE: BaseStatSetter = BaseStatSetter {
    power: Some(4),
    toughness: Some(4),
    own_turn: false,
    condition: Some(StatValue::AtLeast {
        value: &StatValue::SourceCounters(CounterKind::Charge),
        minimum: 10,
    }),
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::CreatureGroupBonus {
                filters: &[CreatureFilter {
                    relation: 0,
                    other: true,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                }],
                stats: Stats {
                    power: 1,
                    toughness: 0,
                },
            }],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Type,
            fragments: &[
                StaticAbilityFragment::SourceCounterCreature {
                    own_turn: false,
                    kind: CounterKind::Charge,
                    minimum: 10,
                    form: SourceCreatureForm::AddCreature,
                    subtypes: &[],
                    keywords: Keywords::FLYING,
                },
                StaticAbilityFragment::BaseStatSetter(0, BASE),
            ],
        },
    ],
    extras: &CardExtras {
        base_stat_setter: Some(BASE),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            event: &TriggerCondition::CreatureDies {
                relation: CastRelation::Any,
                other: false,
                nontoken: false,
                subtype: None,
            },
            guard: StepGuard::SourceNoncreature,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::Charge,
            amount: 1,
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
