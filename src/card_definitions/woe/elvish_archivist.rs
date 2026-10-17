// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 1),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(&TriggerCondition::ArtifactEntersYou),
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 2,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(&TriggerCondition::EnchantmentEntersYou),
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
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
    keywords: Keywords(0),
    ..CardRules::EMPTY
};
