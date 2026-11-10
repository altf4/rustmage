// Direct typed rules from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: true,
        }),
        &[
            Instruction::LoseLifeScaled(
                PlayerSet::Targeted,
                AmountValue::TargetPlayerCreatureCount,
            ),
            Instruction::MoveBattlefield {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                exile: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};
