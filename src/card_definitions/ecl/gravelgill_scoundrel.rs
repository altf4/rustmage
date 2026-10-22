// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    keywords: Keywords::VIGILANCE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::TapPermanents(1),
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 1,
    nonland: false,
    other: true,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
},
                amount: 1,
                optional_skip: Some(1),
            },
            Instruction::GrantKeywords { subject: Subject::Source, keywords: Keywords::CANT_BE_BLOCKED },
        ],
    }],
    ..CardRules::EMPTY
};
