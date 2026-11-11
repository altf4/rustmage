// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
            base: &TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            predicate: &TargetPredicate::Token,
        })),
        &[
            Instruction::CreateCopyTokensScaled {
                subject: Subject::Targets,
                amount: AmountValue::ChosenX,
            },
            Instruction::GrantControlledTokenKeywordsUntilNextTurn(Keywords::INDESTRUCTIBLE),
            Instruction::GainLifeScaled(PlayerSet::Controller, AmountValue::TokensYouControl),
        ],
    ),
    extras: &CardExtras {
        x_symbols: 2,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
