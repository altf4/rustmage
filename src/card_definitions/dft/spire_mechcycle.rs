// Complete Oracle/XMage: permanent animation and a deduplicated live other Mount/Vehicle count.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::HASTE)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::HASTE,
    static_groups: GROUPS,
    activated: &[
        ActivatedAbility {
            exhaust: true,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::FilteredCost {
                        action: CostAction::TapPermanents(1),
                        any: &[CostMatch::Subtype("Mount"), CostMatch::Subtype("Vehicle")],
                    },
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 0,
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
            program: &[
                Instruction::AnimateSource(AnimationSpec {
                    artifact: true,
                    creature_types: AnimationCreatureTypes::Unchanged,
                    base: None,
                    keywords: Keywords::NONE,
                    duration: AnimationDuration::Permanent,
                }),
                Instruction::PutCountersScaled {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: AmountValue::Stat(StatValue::CountUnion(&[
                        CountFilter {
                            zone: CountZone::Battlefield,
                            relation: 0,
                            kind: 0,
                            exclude: 0,
                            subtype: Some("Mount"),
                            basic: false,
                            other: true,
                            cards_only: false,
                        },
                        CountFilter {
                            zone: CountZone::Battlefield,
                            relation: 0,
                            kind: 0,
                            exclude: 0,
                            subtype: Some("Vehicle"),
                            basic: false,
                            other: true,
                            cards_only: false,
                        },
                    ])),
                },
            ],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost::ZERO,
            targets: TargetPlan::NONE,
            program: &[Instruction::Crew { minimum: 2 }],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 5,
            toughness: 4,
        }),
        static_ability_groups: GROUPS,
        exhaust_mask: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
