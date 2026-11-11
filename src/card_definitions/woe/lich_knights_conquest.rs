// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::MaySacrificeAnyNumber(SacrificeFilter {
                kinds: 0,
                other: false,
                keywords: Keywords::NONE,
                source_only: false,
                nonland: false,
                tokens: None,
                artifact_enchantment_or_token: true,
                subtype: None,
            }),
            Instruction::ChooseGraveyardReturnCount(AmountValue::MovedCount),
        ],
    ),
    ..CardRules::EMPTY
};
