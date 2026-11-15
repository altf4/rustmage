// Complete Oracle reviewed; pinned XMage has no class. Both exilers retain
// independent exact face-down play rights while their cards remain exiled.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Player { opponent_only: true }),
        &[
            Instruction::ExileLibraryTopFaceDownPlay {
                library: PlayerSet::Controller,
                beneficiary: PlayerSet::Targeted,
                amount: 1,
                spend_any_mana: true,
            },
            Instruction::ExileLibraryTopFaceDownPlay {
                library: PlayerSet::Targeted,
                beneficiary: PlayerSet::Controller,
                amount: 2,
                spend_any_mana: true,
            },
        ],
    ),
    ..CardRules::EMPTY
};
