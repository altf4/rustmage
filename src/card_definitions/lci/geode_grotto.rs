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
        0, 0, 0, 1, 0, 0,
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
                colored: [0, 0, 0, 1, 0, 0],
                generic: 2,
                mana_component: true,
                ..TAP.cost
            },
            targets: one(TargetRule::Creature),
            program: &[
                Instruction::GrowScaled {
                    subject: Subject::Targets,
                    amount: AmountValue::Stat(StatValue::Count(CountFilter {
                        zone: CountZone::Battlefield,
                        relation: 0,
                        kind: 16,
                        exclude: 0,
                        subtype: None,
                        basic: false,
                        other: false,
                        cards_only: false,
                    })),
                    power: 1,
                    toughness: 0,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::HASTE,
                },
            ],
            sorcery: true,
            ..TAP
        },
    ],
    ..CardRules::EMPTY
};
