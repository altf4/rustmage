// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CreateTokens {
                token: Card::SosSpiritToken,
                amount: 2,
                tapped: false,
            },
            Instruction::IfCastFromNonHand { skip: 1 },
            Instruction::PutPermanentCountersScaled {
                filters: &[BattlefieldFilter {
                    kinds: 0,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: Some("Spirit"),
                    max_mana: u32::MAX,
                }],
                kind: CounterKind::PlusOne,
                amount: AmountValue::Fixed(1),
            },
        ],
    ),
    extras: &CardExtras {
        flashback_cost: Some(ManaCost {
            colored: [2, 0, 0, 0, 0, 0],
            generic: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
