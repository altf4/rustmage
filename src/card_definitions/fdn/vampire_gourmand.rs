// Directly authored from complete pinned Oracle and XMage executable behavior.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Sacrifice,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    keywords: Keywords(0),
                    source_only: false,
                    kinds: 1,
                    nonland: false,
                    other: true,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: Some(2),
            },
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::GrantKeywords {
                subject: Subject::Source,
                keywords: Keywords::CANT_BE_BLOCKED,
            },
        ],
    }],
    ..CardRules::EMPTY
};
