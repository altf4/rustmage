// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ManaSpent {
            minimum: 0,
            increment: false,
            event: &TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::InstantSorcery,
                ordinal: None,
            },
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: false,
        })),
        program: &[
            Instruction::IfManaSpent {
                minimum: None,
                maximum: Some(4),
                skip: 1,
            },
            Instruction::Mill(PlayerSet::Targeted, MillAmount::Fixed(3)),
            Instruction::IfManaSpent {
                minimum: Some(5),
                maximum: None,
                skip: 1,
            },
            Instruction::Mill(PlayerSet::Targeted, MillAmount::Fixed(10)),
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
