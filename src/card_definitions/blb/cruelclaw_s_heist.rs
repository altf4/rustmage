const OPPONENT: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Player {
    opponent_only: true,
}));

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        OPPONENT,
        &[
            Instruction::GiveGift,
            Instruction::RevealTargetHandExileNonlandCastIfGift,
        ],
    ),
    extras: &CardExtras {
        gift: Some(GiftSpec {
            kind: GiftKind::Card,
            token: None,
            promised_targets: OPPONENT,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
