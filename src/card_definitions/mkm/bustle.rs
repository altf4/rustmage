pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ModifyPlayerCreatures {
                players: PlayerSet::Controller,
                power: 2,
                toughness: 2,
                keywords: Keywords::TRAMPLE,
            },
            Instruction::MayTurnControlledCreatureFaceUp,
        ],
    ),
    extras: &CardExtras {
        split_parent: Some(Card::from_catalogue_id(2110)),
        cost_alternatives: &[
            ManaCost {
                colored: [0, 0, 0, 1, 1, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 0, 0, 0, 2, 0],
                generic: 4,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
