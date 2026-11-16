// Complete Oracle behavior, checked against pinned XMage or official release notes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 1,
            max: 2,
        }),
        &[Instruction::DistributeCounters {
            total: 2,
            kind: CounterKind::PlusOne,
        }],
    ),
    extras: &CardExtras {
        beam_me_up_cost: Some(ManaCost {
            generic: 2,
            colored: [1, 0, 0, 0, 0, 0],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
