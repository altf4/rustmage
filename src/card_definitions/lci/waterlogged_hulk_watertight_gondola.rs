const ABILITY: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost::ZERO,
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
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::Mill(
                PlayerSet::Controller,
                MillAmount::Fixed(1),
            )],
            ..ABILITY
        },
        ActivatedAbility {
            cost: ActivationCost {
                colored: [0, 1, 0, 0, 0, 0],
                generic: 3,
                mana_component: true,
                exile_self: true,
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::ExileControlledOrGraveyard,
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 2,
                        nonland: false,
                        other: true,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: Some("Island"),
                    },
                }),
                ..ActivationCost::ZERO
            },
            program: &[Instruction::ReturnCraftedSourceTransformed],
            sorcery: true,
            ..ABILITY
        },
    ],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
