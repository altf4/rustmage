// Oracle and pinned XMage evidence are recorded in the paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Union(&[
            TargetRule::SpellKinds {
                kinds: 12,
                excluded: 0,
            },
            TargetRule::TriggeredAbility { relation: -1 },
        ])),
        &[Instruction::Counter],
    ),
    extras: &CardExtras {
        web_slinging_cost: Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 0,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
