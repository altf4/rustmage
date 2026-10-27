// Directly authored from complete Oracle, pinned Java and actual-return rules.
const OWN: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Creature,
    predicate: &TargetPredicate::OwnedByController,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(OWN),
        &[Instruction::ReturnTargetsCreateNextUpkeep { token: Card(5418) }],
    ),
    extras: &CardExtras {
        kicker_cost: Some(ManaCost {
            colored: [2, 0, 0, 0, 0, 0],
            generic: 2,
        }),
        kicked_targets: Some(TargetPlan::Single(TargetSpec {
            rule: OWN,
            min: 0,
            max: usize::MAX,
        })),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
