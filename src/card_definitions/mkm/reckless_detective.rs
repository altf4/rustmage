pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::SacrificeOrDiscard { nonland: false },
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 16,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: Some(2),
            },
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::Grow {
                subject: Subject::Source,
                power: 2,
                toughness: 0,
            },
        ],
    }],
    ..CardRules::EMPTY
};
