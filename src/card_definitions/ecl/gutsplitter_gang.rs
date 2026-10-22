// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    triggers: &[TriggeredAbility { condition: TriggerCondition::AtStep { step: TriggerStep::Main1, relation: CastRelation::You, guard: StepGuard::Always }, targets: TargetPlan::NONE,
        program: &[Instruction::ChooseObjects { kind: EffectChoiceKind::BlightCost(2), players: PlayerSet::Controller, filter: SacrificeFilter { keywords: Keywords::NONE, source_only: false, kinds: 1, nonland: false, other: false, tokens: None, artifact_enchantment_or_token: false, subtype: None }, amount: 1, optional_skip: Some(1) }, Instruction::Skip(1), Instruction::LoseLife(PlayerSet::Controller, 3)],
    }],
    ..CardRules::EMPTY
};
