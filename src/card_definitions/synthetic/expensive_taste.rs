// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player {
            opponent_only: true,
        }),
        &[Instruction::ExileLibraryTopFaceDownPlay {
            library: PlayerSet::Targeted,
            beneficiary: PlayerSet::Controller,
            amount: 2,
            spend_any_mana: false,
        }],
    ),
    extras: &CardExtras {
        adventure_parent: Some(Card(1011)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
