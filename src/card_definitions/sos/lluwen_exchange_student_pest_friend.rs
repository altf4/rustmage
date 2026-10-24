// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3,4),
    extras: &CardExtras { prepared_spell: Some(Card(5373)), enters_prepared: true, ..CardExtras::EMPTY },
    activated: &[ActivatedAbility { exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield, cycling: false,
        cost: ActivationCost { colored: [0;6], generic: 0, alternatives: &[], additional: Some(AdditionalCost { kind: EffectChoiceKind::ExileGraveyard, filter: SacrificeFilter { keywords: Keywords::NONE, source_only: false, kinds: 1, nonland: false, other: false, tokens: None, artifact_enchantment_or_token: false, subtype: None } }), mana_component: false, tap: false, sacrifice: false, exile_self: false, discard: false, life: 0, remove: None, loyalty: None }, targets: TargetPlan::NONE, program: &[Instruction::PrepareSource], sorcery: true, once_per_turn: false }],
    ..CardRules::EMPTY
};
