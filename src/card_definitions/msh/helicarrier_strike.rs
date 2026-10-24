// Directly authored from the full pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Filtered {
            base: &TargetRule::Creature,
            predicate: &TargetPredicate::AttackingOrBlocking,
        }),
        &[
            Instruction::IfTeamwork { skip: 2 },
            Instruction::Damage(4),
            Instruction::Skip(1),
            Instruction::Damage(2),
        ],
    ),
    extras: &CardExtras {
        teamwork: Some(TeamworkSpec {
            minimum: 2,
            paid_targets: None,
            normal_modes: 0,
            paid_modes: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
