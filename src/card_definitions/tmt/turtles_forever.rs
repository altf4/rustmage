// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::SearchLibraryOrOutsideLegendarySplit {
            maximum: 4,
            opponent_choices: 2,
        }],
    ),
    ..CardRules::EMPTY
};
