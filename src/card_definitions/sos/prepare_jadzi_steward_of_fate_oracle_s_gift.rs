// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CreateTokensScaled {
                token: Card::SosFractalToken,
                amount: AmountValue::ChosenX,
                tapped: false,
            },
            Instruction::PutPermanentCountersScaled {
                filters: &[BattlefieldFilter {
                    kinds: 0,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: Some("Fractal"),
                    max_mana: u32::MAX,
                }],
                kind: CounterKind::PlusOne,
                amount: AmountValue::ChosenX,
            },
        ],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/jadzi_steward_of_fate_oracle_s_gift"),
        x_symbols: 2,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
