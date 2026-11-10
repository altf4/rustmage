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
        0, 0, 0, 0, 1, 0,
    ]))],
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    exhaust: false,
    cycling: false,
    sorcery: false,
    once_per_turn: false,
};

const CREATURES: StatValue = StatValue::Count(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 1,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
});
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::Always,
    activated: &[
        BASE_MANA,
        STATION,
        ActivatedAbility {
            cost: ActivationCost {
                colored: [0, 0, 0, 0, 1, 0],
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::AddMana(ManaProduction::ChoiceStat {
                colors: 16,
                amount: &CREATURES,
            })],
            guard: ActivationGuard::SourceCounterAtLeast {
                kind: CounterKind::Charge,
                minimum: 12,
            },
            ..BASE_MANA
        },
    ],
    ..CardRules::EMPTY
};
