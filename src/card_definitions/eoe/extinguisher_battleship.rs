const STATION: ActivatedAbility = ActivatedAbility {
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
    exhaust: false,
    cycling: false,
    sorcery: true,
    once_per_turn: false,
};

const BASE: BaseStatSetter = BaseStatSetter {
    power: Some(10),
    toughness: Some(10),
    own_turn: false,
    condition: Some(StatValue::AtLeast {
        value: &StatValue::SourceCounters(CounterKind::Charge),
        minimum: 5,
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
                minimum: 5,
                form: SourceCreatureForm::AddCreature,
                subtypes: &[],
                keywords: Keywords::FLYING.union(Keywords::TRAMPLE),
            },
            StaticAbilityFragment::BaseStatSetter(0, BASE),
        ],
    }],
    extras: &CardExtras {
        base_stat_setter: Some(BASE),
        ..CardExtras::EMPTY
    },
    activated: &[STATION],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Filtered {
            base: &object(0),
            predicate: &TargetPredicate::WithoutKinds(1),
        }),
        program: &[
            Instruction::Destroy,
            Instruction::DamagePlayerCreatures {
                players: PlayerSet::Each,
                amount: 4,
            },
        ],
    }],
    ..CardRules::EMPTY
};
