// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Sacrificed {
            filter: SacrificeFilter {
                artifact_enchantment_or_token: false,
                keywords: Keywords(0),
                source_only: false,
                kinds: 16,
                nonland: false,
                other: false,
                tokens: None,
                subtype: None,
            },
            own_turn: false,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::AddMana(ManaProduction::Fixed([0, 0, 0, 1, 0, 0])),
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
