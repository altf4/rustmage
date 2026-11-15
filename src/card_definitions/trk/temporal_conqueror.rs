// Oracle and official Star Trek release notes reviewed; XMage class absent at the pinned revision.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ReturnGraveyardToHandMatching(SearchFilter::Land),
                players: PlayerSet::Controller,
                filter: SacrificeFilter { keywords: Keywords::NONE, source_only: false, kinds: 0, nonland: false, other: false, tokens: None, artifact_enchantment_or_token: false, subtype: None },
                amount: 1,
                optional_skip: None,
            },
            Instruction::IfAmountAtLeast { value: AmountValue::MovedCount, minimum: 1, skip: 1 },
            Instruction::Skip(1),
            Instruction::CreateTokens { token: Card(5302), amount: 1, tapped: false },
        ],
    }],
    ..CardRules::EMPTY
};
