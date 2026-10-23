// Direct typed behavior from complete pinned Oracle/XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::DamagePlayerCreatures {
            players: PlayerSet::Each,
            amount: 2,
        }],
    ),
    extras: &CardExtras {
        sneak_costs: &[ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 0,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
