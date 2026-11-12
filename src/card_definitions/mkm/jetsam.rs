pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Mill(PlayerSet::Opponent, MillAmount::Fixed(3)),
            Instruction::CastOpponentGraveyardFreeThenExile,
        ],
    ),
    extras: &CardExtras {
        split_parent: Some(Card::from_catalogue_id(1566)),
        cost_alternatives: &[
            ManaCost {
                colored: [0, 1, 1, 0, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 0, 2, 0, 0, 0],
                generic: 4,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
