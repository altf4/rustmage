// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Emblem,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LifeGained {
            first_only: false,
            own_turn: false,
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        program: &[Instruction::LoseGainedLife(PlayerSet::Targeted)],
    }],
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
    deck_limit: None,
    ..CardRules::EMPTY
};
