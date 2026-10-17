// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::NONE),
        &[
            Instruction::GiveGift,
            Instruction::PlayerHexproofThisTurn,
            Instruction::GrantPermanentsThisTurn(Keywords::HEXPROOF),
            Instruction::IfGift {
                promised: true,
                skip: 1,
            },
            Instruction::GrantPermanentsThisTurn(Keywords::INDESTRUCTIBLE),
        ],
    ),
    triggers: &[],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NONE,
    extras: &CardExtras {
        gift: {
            const SPEC: GiftSpec = GiftSpec {
                kind: GiftKind::Card,
                token: None,
                promised_targets: TargetPlan::Single(TargetSpec::NONE),
            };
            Some(SPEC)
        },
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
