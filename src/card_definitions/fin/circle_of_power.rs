const WIZARDS: &[BattlefieldFilter] = &[BattlefieldFilter {
    kinds: 0,
    relation: 0,
    nonland: false,
    tapped: None,
    without: Keywords::NONE,
    subtype: Some("Wizard"),
    max_mana: u32::MAX,
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::LoseLife(PlayerSet::Controller, 2),
            Instruction::CreateTokens {
                token: Card(5531),
                amount: 1,
                tapped: false,
            },
            Instruction::GrowPermanentGroup {
                filters: WIZARDS,
                power: 1,
                toughness: 0,
            },
            Instruction::GrantPermanentGroupKeywords {
                filters: WIZARDS,
                keywords: Keywords::LIFELINK,
            },
        ],
    ),
    ..CardRules::EMPTY
};
