const OTHER_ARTIFACT_OR_CREATURE: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 1 | 16,
    nonland: false,
    other: true,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};

const TAP: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[],
    sorcery: false,
    once_per_turn: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::Sacrifice,
                    filter: OTHER_ARTIFACT_OR_CREATURE,
                }),
                ..TAP.cost
            },
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
            sorcery: true,
            ..TAP
        },
        ActivatedAbility {
            cost: ActivationCost {
                generic: 2,
                mana_component: true,
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::DiscardHand,
                    filter: SacrificeFilter {
                        kinds: 0,
                        other: false,
                        ..OTHER_ARTIFACT_OR_CREATURE
                    },
                }),
                ..TAP.cost
            },
            program: &[Instruction::TransformSource],
            ..TAP
        },
    ],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
