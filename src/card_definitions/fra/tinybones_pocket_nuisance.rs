// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::Discard(PlayerSet::Opponent, 1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::Discarded {
                relation: CastRelation::Any,
                one_or_more: true,
            },
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::DamagePlayers(PlayerSet::Opponent, 1)],
        },
    ],
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
