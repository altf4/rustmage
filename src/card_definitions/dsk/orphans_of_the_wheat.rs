pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::TapAnyUntappedPermanents,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    kinds: 1,
                    keywords: Keywords::NONE,
                    source_only: false,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 0,
                optional_skip: None,
            },
            Instruction::GrowScaled {
                subject: Subject::Source,
                amount: AmountValue::RememberedCount,
                power: 1,
                toughness: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
