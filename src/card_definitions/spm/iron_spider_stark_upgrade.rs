use std::num::NonZeroU16;

const ARTIFACT_CREATURE_OR_VEHICLE: &[ControlledPermanentFilter] = &[
    ControlledPermanentFilter {
        all_kinds: 1 | 16,
        ..ControlledPermanentFilter::ANY
    },
    ControlledPermanentFilter {
        subtype: Some("Vehicle"),
        ..ControlledPermanentFilter::ANY
    },
];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::VIGILANCE,
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::ChangeControlledPermanentGroupCounters {
                filters: ARTIFACT_CREATURE_OR_VEHICLE,
                effect: PermanentGroupCounters::Add {
                    kind: CounterKind::PlusOne,
                    amount: 1,
                },
            }],
            ..ACTIVATION
        },
        ActivatedAbility {
            cost: ActivationCost {
                generic: 2,
                mana_component: true,
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::RemovePermanentCounters {
                        amount: NonZeroU16::new(2).unwrap(),
                        kind: CounterKind::PlusOne,
                        bearer_kinds: 16,
                    },
                    filter: SacrificeFilter {
                        kinds: 0,
                        source_only: false,
                        other: false,
                        nonland: false,
                        tokens: None,
                        subtype: None,
                        keywords: Keywords::NONE,
                        artifact_enchantment_or_token: false,
                    },
                }),
                ..ActivationCost::ZERO
            },
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
            ..ACTIVATION
        },
    ],
    ..CardRules::EMPTY
};

const ACTIVATION: ActivatedAbility = ActivatedAbility {
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    exhaust: false,
    once_per_turn: false,
    sorcery: false,
    cost: ActivationCost::ZERO,
    targets: TargetPlan::NONE,
    program: &[],
};
