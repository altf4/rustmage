// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5316),
                amount: 3,
                tapped: false,
            },
            Instruction::GrantPermanentGroupKeywords {
                filters: &[BattlefieldFilter {
                    kinds: 0,
                    relation: 0,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: Some("Gremlin"),
                    max_mana: u32::MAX,
                }],
                keywords: Keywords::MENACE
                    .union(Keywords::LIFELINK)
                    .union(Keywords::HASTE),
            },
        ],
    ),
    ..CardRules::EMPTY
};
