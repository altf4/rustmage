const TAP_MANA: ActivatedAbility = ActivatedAbility {
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    sorcery: false,
    once_per_turn: false,
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Fixed([
        0, 0, 0, 0, 0, 1,
    ]))],
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::Always,
    activated: &[
        TAP_MANA,
        ActivatedAbility {
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
                ..TAP_MANA.cost
            },
            program: &[Instruction::AddMana(ManaProduction::Choice {
                mask: 31,
                amount: 1,
            })],
            ..TAP_MANA
        },
        ActivatedAbility {
            cost: ActivationCost {
                generic: 2,
                mana_component: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
            ..TAP_MANA
        },
    ],
    ..CardRules::EMPTY
};
