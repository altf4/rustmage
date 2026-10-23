// Direct typed behavior from full pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ReturnBattlefield {
                filters: &[BattlefieldFilter {
                    kinds: 1,
                    relation: -1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                attacking: false,
                players: None,
            },
            Instruction::ChooseWheel { draw: 7 },
            Instruction::ShuffleWheelHandsAndGraveyards,
            Instruction::Draw(PlayerSet::Wheelers, 7),
            Instruction::ExileResolvingSpell,
        ],
    ),
    ..CardRules::EMPTY
};
