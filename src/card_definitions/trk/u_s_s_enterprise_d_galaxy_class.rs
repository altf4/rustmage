// Complete Oracle and pinned USSEnterpriseDGalaxyClass.java reviewed. The
// source-incarnation trigger fires on the first actual charge addition each turn.
const BASE: BaseStatSetter = BaseStatSetter {
    power: Some(4),
    toughness: Some(5),
    own_turn: false,
    condition: Some(StatValue::AtLeast {
        value: &StatValue::SourceCounters(CounterKind::Charge),
        minimum: 7,
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
                minimum: 7,
                form: SourceCreatureForm::AddCreature,
                subtypes: &[],
                keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
            },
            StaticAbilityFragment::BaseStatSetter(0, BASE),
        ],
    }],
    extras: &CardExtras {
        base_stat_setter: Some(BASE),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FirstSelfCountersAddedThisTurn {
            kind: CounterKind::Charge,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileTopPlay {
            amount: 1,
            next_turn: false,
        }],
    }],
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
