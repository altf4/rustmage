// Whole-card typed Oracle behavior; full XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::LoseHalfLife(PlayerSet::Each),
            Instruction::ChooseHalf {
                kind: EffectChoiceKind::Discard,
            },
            Instruction::ChooseHalf {
                kind: EffectChoiceKind::Sacrifice,
            },
        ],
    ),
    ..CardRules::EMPTY
};
