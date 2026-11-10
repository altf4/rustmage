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

const BASE_MANA: ActivatedAbility = ActivatedAbility {
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Fixed([
        0, 0, 0, 1, 0, 0,
    ]))],
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    exhaust: false,
    cycling: false,
    sorcery: false,
    once_per_turn: false,
};

const YOUR_CREATURES: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::Always,
    activated: &[
        BASE_MANA,
        STATION,
        ActivatedAbility {
            cost: ActivationCost {
                generic: 1,
                colored: [0, 0, 0, 1, 0, 0],
                mana_component: true,
                tap: true,
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::Sacrifice,
                    filter: SacrificeFilter {
                        kinds: 2,
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
            program: &[
                Instruction::CreateTokens {
                    token: Card(5309),
                    amount: 1,
                    tapped: false,
                },
                Instruction::GrowCreatures {
                    filter: YOUR_CREATURES,
                    power: 1,
                    toughness: 0,
                },
                Instruction::GrantCreatures {
                    filter: YOUR_CREATURES,
                    keywords: Keywords::HASTE,
                },
            ],
            guard: ActivationGuard::SourceCounterAtLeast {
                kind: CounterKind::Charge,
                minimum: 12,
            },
            ..BASE_MANA
        },
    ],
    ..CardRules::EMPTY
};
