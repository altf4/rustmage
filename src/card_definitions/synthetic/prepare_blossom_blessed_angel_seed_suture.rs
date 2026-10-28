// Seed Suture prepared by Blossom-Blessed Angel.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
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
            colored: [0, 0, 0, 0, 1, 0],
            generic: 0,
        }],
        prepared_identity: Some("@prepare/blossom_blessed_angel_seed_suture"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
