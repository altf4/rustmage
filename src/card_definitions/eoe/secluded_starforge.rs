const BASE_MANA: ActivatedAbility = ActivatedAbility {
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Fixed([
        0, 0, 0, 0, 0, 1,
    ]))],
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    exhaust: false,
    cycling: false,
    sorcery: false,
    once_per_turn: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        BASE_MANA,
        ActivatedAbility {
            cost: ActivationCost {
                generic: 2,
                mana_component: true,
                tap: true,
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::TapXPermanents,
                    filter: SacrificeFilter {
                        kinds: 16,
                        other: false,
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
            targets: one(TargetRule::Creature),
            program: &[Instruction::GrowScaled {
                subject: Subject::Targets,
                amount: AmountValue::ChosenX,
                power: 1,
                toughness: 0,
            }],
            sorcery: true,
            ..BASE_MANA
        },
        ActivatedAbility {
            cost: ActivationCost {
                generic: 5,
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::CreateTokens {
                token: Card(5309),
                amount: 1,
                tapped: false,
            }],
            ..BASE_MANA
        },
    ],
    ..CardRules::EMPTY
};
