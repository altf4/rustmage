// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2,2),
    triggers: &[TriggeredAbility { condition: TriggerCondition::SpellCast { relation: CastRelation::You, filter: CastFilter::InstantSorcery, ordinal: None }, targets: TargetPlan::NONE, program: &[
        Instruction::ChooseObjects { kind: EffectChoiceKind::TapPermanents(3), players: PlayerSet::Controller, filter: SacrificeFilter { keywords: Keywords::NONE, source_only: false, kinds: 1, nonland: false, other: false, tokens: None, artifact_enchantment_or_token: false, subtype: None }, amount: 3, optional_skip: Some(1) },
        Instruction::CopyCastSpell { prior_spells: false },
    ] }],
    ..CardRules::EMPTY
};
