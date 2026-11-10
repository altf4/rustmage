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
        1, 0, 0, 0, 0, 0,
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
    entry: EntryTapped::Always,
    activated: &[
        BASE_MANA,
        STATION,
        ActivatedAbility {
            cost: ActivationCost {
                generic: 3,
                colored: [1, 0, 0, 0, 0, 0],
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 16 | 32,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::CreateLegendaryCopyTokens {
                subject: Subject::Targets,
                amount: 1,
            }],
            guard: ActivationGuard::SourceCounterAtLeast {
                kind: CounterKind::Charge,
                minimum: 12,
            },
            sorcery: true,
            ..BASE_MANA
        },
    ],
    ..CardRules::EMPTY
};
