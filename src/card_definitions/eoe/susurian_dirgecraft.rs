const BASE: BaseStatSetter = BaseStatSetter {
    power: Some(4),
    toughness: Some(3),
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
        targets: TargetPlan::NONE,
        program: &[Instruction::ChooseObjects {
            kind: EffectChoiceKind::Sacrifice,
            players: PlayerSet::Opponent,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 1,
                nonland: false,
                other: false,
                tokens: Some(false),
                artifact_enchantment_or_token: false,
                subtype: None,
            },
            amount: 1,
            optional_skip: None,
        }],
    }],
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
