// Direct typed behavior from the pinned Oracle and complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 7),
    keywords: Keywords::NONE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects { kind: EffectChoiceKind::BlightCost(2), players: PlayerSet::Controller, filter: SacrificeFilter { keywords: Keywords::NONE, source_only: false, kinds: 1, nonland: false, other: false, tokens: None, artifact_enchantment_or_token: false, subtype: None }, amount: 1, optional_skip: Some(2) },
            Instruction::Draw(PlayerSet::Controller, 1),Instruction::LoseLife(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
