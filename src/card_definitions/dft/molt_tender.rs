// The second activation is a mana ability with its tap and graveyard exile paid together.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Mill(
                PlayerSet::Controller,
                MillAmount::Fixed(1),
            )],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                tap: true,
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::ExileGraveyard,
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 0,
                        nonland: false,
                        other: false,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: None,
                    },
                }),
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Choice {
                mask: 31,
                amount: 1,
            })],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
