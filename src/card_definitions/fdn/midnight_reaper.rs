// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDies {
            relation: CastRelation::You,
            other: false,
            nontoken: true,
            subtype: None,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::DamagePlayers(PlayerSet::Controller, 1),
            Instruction::Draw(PlayerSet::Controller, 1),
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
