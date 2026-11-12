// Directly authored from the paired Oracle and pinned XMage sources.
const CREATURES: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    activated: &[
        ActivatedAbility {
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
            program: &[
                Instruction::GrantKeywords {
                    subject: Subject::Source,
                    keywords: Keywords::INDESTRUCTIBLE,
                },
                Instruction::TapSource,
            ],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [0, 0, 0, 0, 2, 0],
                generic: 3,
                mana_component: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GrowCreaturesScaled {
                filter: CREATURES,
                excluding_subtype: None,
                amount: AmountValue::GreatestCreaturePower(CREATURES),
                power: 1,
                toughness: 1,
            }],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
