// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 2,
            },
            Instruction::If {
                guard: StepGuard::GainedLife,
                skip: 1,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords(8448),
            },
        ],
    ),
    triggers: &[],
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
