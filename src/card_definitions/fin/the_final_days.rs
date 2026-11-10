// Direct typed behavior from the paired Oracle and pinned XMage sources.
const HORROR: Card = Card(5536);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfCastFromGraveyard { skip: 2 },
            Instruction::CreateTokensScaled {
                token: HORROR,
                amount: AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Graveyard,
                    relation: 0,
                    kind: 1,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: true,
                })),
                tapped: true,
            },
            Instruction::Skip(1),
            Instruction::CreateTokens {
                token: HORROR,
                amount: 2,
                tapped: true,
            },
        ],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            generic: 4,
            colored: [0, 0, 2, 0, 0, 0],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
