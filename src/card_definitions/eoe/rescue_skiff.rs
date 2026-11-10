const BASE: BaseStatSetter = BaseStatSetter {
    power: Some(5),
    toughness: Some(6),
    own_turn: false,
    condition: Some(StatValue::AtLeast {
        value: &StatValue::SourceCounters(CounterKind::Charge),
        minimum: 10,
    }),
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 2,
    first_layer: AbilityLayer::Type,
    fragments: &[
        StaticAbilityFragment::SourceCounterCreature {
            own_turn: false,
            kind: CounterKind::Charge,
            minimum: 10,
            form: SourceCreatureForm::ArtifactCreature,
            subtypes: &[],
            keywords: Keywords::FLYING,
        },
        StaticAbilityFragment::BaseStatSetter(0, BASE),
    ],
}];

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
        targets: one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1 | 32,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::ReturnToBattlefield],
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
