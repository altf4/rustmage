// Adventure face of Elusive Otter; source and Oracle pinned on its parent.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::creature_you_control(),
            min: 0,
            max: usize::MAX,
        }),
        &[Instruction::DistributeCountersX {
            kind: CounterKind::PlusOne,
        }],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(1294)),
        x_symbols: 1,
        targets_up_to_x: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
