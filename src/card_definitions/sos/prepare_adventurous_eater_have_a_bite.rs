// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    ),
    extras: &CardExtras {
        prepared_identity: Some("@prepare/adventurous_eater_have_a_bite"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
