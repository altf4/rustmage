// Direct typed behavior from the pinned Oracle and official release notes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: true,
        }),
        &[
            Instruction::IfWaterbendPaid { skip: 2 },
            Instruction::ControlTargetPlayerNextTurn,
            Instruction::Skip(1),
            Instruction::ControlTargetPlayerNextCombat,
            Instruction::ExileResolvingSpell,
        ],
    ),
    extras: &CardExtras {
        waterbend_additional: Some(WaterbendAdditional::Optional(10)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
