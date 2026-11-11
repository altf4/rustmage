const BASE_ACTIVATION: ActivatedAbility = ActivatedAbility {
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cost: ActivationCost::ZERO,
    targets: TargetPlan::NONE,
    program: &[],
    cycling: false,
    exhaust: false,
    sorcery: false,
    once_per_turn: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::VIGILANCE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::VIGILANCE,
        )],
    }],
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                generic: 3,
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
            ..BASE_ACTIVATION
        },
        ActivatedAbility {
            cost: ActivationCost {
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::TapPermanents(2),
                    filter: SacrificeFilter {
                        kinds: 0,
                        keywords: Keywords::NONE,
                        source_only: false,
                        other: false,
                        nonland: false,
                        tokens: Some(true),
                        artifact_enchantment_or_token: false,
                        subtype: None,
                    },
                }),
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AnimateSource(AnimationSpec {
                artifact: true,
                creature_types: AnimationCreatureTypes::Unchanged,
                base: None,
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            })],
            ..BASE_ACTIVATION
        },
    ],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 6,
            toughness: 6,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
