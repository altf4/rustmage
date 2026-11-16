// Complete Oracle and official Star Trek release-note behavior; no pinned XMage class exists.
const ARTIFACT_CREATURES: CountFilter = CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 1 | 16,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
};
const AFFINITY: SpellReduction = SpellReduction {
    source_only: true,
    filter: CastFilter::Any,
    amount: SpellReductionAmount::Stat {
        value: StatValue::CountAllKinds(ARTIFACT_CREATURES),
        factor: 1,
    },
    turn: CostTurn::Any,
    guard: StepGuard::Always,
};
const STATIONED: StatValue = StatValue::AtLeast {
    value: &StatValue::SourceCounters(CounterKind::Charge),
    minimum: 12,
};
const BASE: BaseStatSetter = BaseStatSetter {
    power: Some(6),
    toughness: Some(6),
    own_turn: false,
    condition: Some(STATIONED),
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CostReduction(0, AFFINITY)],
    },
    StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::SourceCounterCreature {
                own_turn: false,
                kind: CounterKind::Charge,
                minimum: 12,
                form: SourceCreatureForm::AddCreature,
                subtypes: &[],
                keywords: Keywords::NONE,
            },
            StaticAbilityFragment::BaseStatSetter(0, BASE),
        ],
    },
    StaticAbilityGroup {
        line: 3,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::ControlledPermanentAllKindsKeywordsAtSourceCounters {
                kinds: 1 | 16,
                kind: CounterKind::Charge,
                minimum: 12,
                keywords: Keywords::FLYING,
            },
        ],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GROUPS,
    extras: &CardExtras {
        base_stat_setter: Some(BASE),
        cost_reductions: &[AFFINITY],
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: 1,
        }),
        program: &[Instruction::AssimilateLinkedExileOrExileTarget],
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
