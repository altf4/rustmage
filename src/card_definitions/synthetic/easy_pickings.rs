// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::NONE),
        &[Instruction::DamageBattlefield {
            filters: &[BattlefieldFilter {
                kinds: 1,
                relation: 1,
                nonland: false,
                tapped: None,
                without: Keywords::NONE,
                subtype: None,
                max_mana: 4294967295,
            }],
            players: None,
            amount: 1,
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
    extras: &CardExtras {
        adventure_parent: Some(Card(1792)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
