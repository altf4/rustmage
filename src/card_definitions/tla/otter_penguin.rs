// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CardDrawn {
            relation: CastRelation::You,
            ordinal: Some(2),
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::Grow {
                subject: Subject::Source,
                power: 1,
                toughness: 2,
            },
            Instruction::GrantKeywords {
                subject: Subject::Source,
                keywords: Keywords(16384),
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
