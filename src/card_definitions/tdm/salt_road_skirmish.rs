// Source-audited direct typed definition; see paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        &[
            Instruction::Destroy,
            Instruction::CreateTemporaryTokens {
                token: Card(5255),
                amount: AmountValue::Fixed(2),
                keywords: Keywords::HASTE,
            },
        ],
    ),
    ..CardRules::EMPTY
};
