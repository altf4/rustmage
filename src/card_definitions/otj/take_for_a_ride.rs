pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::GainControlUntilEndTurn,
            Instruction::Untap,
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::HASTE,
            },
        ],
    ),
    extras: &CardExtras {
        self_cast_timing: Some(SelfCastTiming::FlashAfterCrime),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
