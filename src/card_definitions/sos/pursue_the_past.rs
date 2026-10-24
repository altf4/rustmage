// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(TargetPlan::NONE, &[Instruction::GainLife(PlayerSet::Controller,2), Instruction::ChooseObjects { kind: EffectChoiceKind::Discard, players: PlayerSet::Controller, filter: SacrificeFilter { keywords: Keywords::NONE, source_only: false, kinds: 0, nonland: false, other: false, tokens: None, artifact_enchantment_or_token: false, subtype: None }, amount: 1, optional_skip: Some(1) }, Instruction::Draw(PlayerSet::Controller,2)]),
    extras: &CardExtras { flashback_cost: Some(ManaCost { colored: [1,0,0,1,0,0], generic: 2 }), ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
