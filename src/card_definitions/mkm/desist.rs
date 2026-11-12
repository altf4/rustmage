pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::MoveBattlefield {
            filters: &[BattlefieldFilter {
                kinds: 16 | 32,
                relation: -1,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: u32::MAX,
            }],
            exile: false,
        }],
    ),
    extras: &CardExtras {
        split_parent: Some(Card::from_catalogue_id(692)),
        cost_alternatives: &[
            ManaCost {
                colored: [1, 0, 0, 0, 1, 0],
                generic: 4,
            },
            ManaCost {
                colored: [2, 0, 0, 0, 0, 0],
                generic: 4,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
