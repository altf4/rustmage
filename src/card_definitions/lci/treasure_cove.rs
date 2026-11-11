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
    program: &[Instruction::AddMana(ManaProduction::Fixed([
        0, 0, 0, 0, 0, 1,
    ]))],
    sorcery: false,
    once_per_turn: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[
        TAP,
        ActivatedAbility {
            cost: ActivationCost {
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::Sacrifice,
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 0,
                        nonland: false,
                        other: false,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: Some("Treasure"),
                    },
                }),
                ..TAP.cost
            },
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
            ..TAP
        },
    ],
    ..CardRules::EMPTY
};
