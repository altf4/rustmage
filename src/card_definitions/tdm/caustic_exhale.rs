// Directly authored from the pinned Oracle and complete XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[Instruction::Grow {
            subject: Subject::Targets,
            power: -3,
            toughness: -3,
        }],
    ),
    extras: &CardExtras {
        behold_cast: Some(BeholdCastSpec {
            subtype: "Dragon",
            decline_generic: 1,
            grants_flash: false,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
