// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::IfWaterbendPaid { skip: 4 },
            Instruction::ShuffleGraveyardIntoLibrary,
            Instruction::Draw(PlayerSet::Controller, 7),
            Instruction::NoMaximumHandSize,
            Instruction::Skip(1),
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::ExileResolvingSpell,
        ],
    ),
    extras: &CardExtras {
        waterbend_additional: Some(WaterbendAdditional::Optional(6)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
