// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::creature_you_control(),
            min: 0,
            max: 3,
        }),
        &[Instruction::DistributeCounters {
            total: 3,
            kind: CounterKind::PlusOne,
        }],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card::from_catalogue_id(3156)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
