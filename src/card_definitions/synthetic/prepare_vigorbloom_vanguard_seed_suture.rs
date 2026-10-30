// Seed Suture prepared by Vigorbloom Vanguard; gated pending validation.
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
        cost_alternatives: &[ManaCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 0,
        }],
        prepared_identity: Some("@prepare/vigorbloom_vanguard_seed_suture"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
