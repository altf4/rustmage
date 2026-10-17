// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LandEntersYou,
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::May { skip: 1 },
            Instruction::SetBaseStats {
                subject: Subject::Source,
                power: Some(4),
                toughness: Some(2),
            },
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
    keywords: Keywords(0),
    ..CardRules::EMPTY
};
