const fn tap_tokens(amount: u32) -> ActivationCost {
    ActivationCost {
        additional: Some(AdditionalCost {
            kind: EffectChoiceKind::TapPermanents(amount),
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 0,
                nonland: false,
                other: false,
                tokens: Some(true),
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        }),
        ..ActivationCost::ZERO
    }
}
const BASE: ActivatedAbility = ActivatedAbility {
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
    effect: Effect::Creature(4, 3),
    activated: &[
        ActivatedAbility {
            cost: tap_tokens(2),
            program: &[Instruction::AddMana(ManaProduction::any_color(1))],
            ..BASE
        },
        ActivatedAbility {
            cost: tap_tokens(3),
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
            ..BASE
        },
        ActivatedAbility {
            cost: tap_tokens(4),
            program: &[
                Instruction::PutCounters {
                    subject: Subject::Source,
                    kind: CounterKind::PlusOne,
                    amount: 3,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Source,
                    keywords: Keywords::TRAMPLE,
                },
            ],
            ..BASE
        },
    ],
    ..CardRules::EMPTY
};
