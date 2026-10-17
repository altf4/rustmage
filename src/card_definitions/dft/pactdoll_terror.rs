// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentEntersYou {
            kinds: 16,
            nontoken: false,
            subtype: None,
            include_source: true,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::LoseLife(PlayerSet::Opponent, 1),
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
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
    ..CardRules::EMPTY
};
