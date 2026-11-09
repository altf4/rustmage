// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::MoveBattlefield {
                filters: &[BattlefieldFilter {
                    kinds: 1 | 16,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                exile: false,
            },
            Instruction::EndTurn,
        ],
    ),
    ..CardRules::EMPTY
};
