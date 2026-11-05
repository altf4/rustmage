// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Creature,
            predicate: &TargetPredicate::AttackingOrBlocking,
        })),
        &[
            Instruction::Damage(5),
            Instruction::IfBeheld {
                beheld: true,
                skip: 1,
            },
            Instruction::GainLife(PlayerSet::Controller, 2),
        ],
    ),
    extras: &CardExtras {
        behold_cast: Some(BeholdCastSpec {
            subtype: "Dragon",
            decline_generic: 0,
            grants_flash: false,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
