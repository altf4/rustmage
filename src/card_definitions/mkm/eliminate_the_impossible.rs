// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5296),
                amount: 1,
                tapped: false,
            },
            Instruction::GrowPermanentGroup {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: 1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                power: -2,
                toughness: 0,
            },
            Instruction::UnsuspectCreatures {
                filter: CreatureFilter {
                    relation: 1,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
            },
        ],
    ),
    ..CardRules::EMPTY
};
