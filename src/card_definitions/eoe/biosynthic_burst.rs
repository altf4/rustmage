// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::creature_you_control()),
        &[
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::REACH
                    .union(Keywords::TRAMPLE)
                    .union(Keywords::INDESTRUCTIBLE),
            },
            Instruction::Untap,
        ],
    ),
    ..CardRules::EMPTY
};
