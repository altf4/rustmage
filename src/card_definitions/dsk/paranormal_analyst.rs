pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::YouManifestDread,
        targets: TargetPlan::NONE,
        program: &[Instruction::ChooseObjects {
            kind: EffectChoiceKind::ReturnManifestGraveyardToHand,
            players: PlayerSet::Controller,
            filter: SacrificeFilter {
                kinds: 0,
                keywords: Keywords::NONE,
                source_only: false,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
            amount: 1,
            optional_skip: None,
        }],
    }],
    ..CardRules::EMPTY
};
