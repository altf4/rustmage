// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::creature_you_control()),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 1,
            },
            Instruction::RestrictBlockerPower {
                group: 0,
                minimum: 3,
            },
        ],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(731)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
