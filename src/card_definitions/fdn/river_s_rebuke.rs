// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: false,
        })),
        &[Instruction::ReturnBattlefield {
            filters: &[BattlefieldFilter {
                kinds: 0,
                relation: -1,
                nonland: true,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: 4294967295,
            }],
            attacking: false,
            players: Some(PlayerSet::Targeted),
        }],
    ),
    triggers: &[],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::NONE,
    ..CardRules::EMPTY
};
