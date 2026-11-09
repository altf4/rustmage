// Direct typed behavior; complete Oracle body and pinned XMage sources reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::Exile,
            Instruction::CreateTokensFor {
                players: PlayerSet::TargetController,
                token: Card(5493),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    keywords: Keywords::CHANGELING,
    ..CardRules::EMPTY
};
