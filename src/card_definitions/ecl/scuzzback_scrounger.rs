// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::NONE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep { step: TriggerStep::Main1, relation: CastRelation::You, guard: StepGuard::Always },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects { kind: EffectChoiceKind::BlightCost(1), players: PlayerSet::Controller, filter: SacrificeFilter { keywords: Keywords::NONE, source_only: false, kinds: 1, nonland: false, other: false, tokens: None, artifact_enchantment_or_token: false, subtype: None }, amount: 1, optional_skip: Some(1) },
            Instruction::CreateTokens { token: Card(5302), amount: 1, tapped: false },
        ],
    }],
    ..CardRules::EMPTY
};
