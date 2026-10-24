// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2,1),
    keywords: Keywords::FLYING.union(Keywords::HASTE),
    triggers: &[TriggeredAbility { condition: TriggerCondition::CardsLeaveGraveyard, targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())), program: &[Instruction::PutCounters { subject: Subject::Targets, kind: CounterKind::PlusOne, amount: 1 }] }],
    activated: &[ActivatedAbility {
        exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield, cycling: false,
        cost: ActivationCost { colored: [0;6], generic: 0, alternatives: &[], additional: Some(AdditionalCost { kind: EffectChoiceKind::Discard, filter: SacrificeFilter { keywords: Keywords::NONE, source_only: false, kinds: 0, nonland: false, other: false, tokens: None, artifact_enchantment_or_token: false, subtype: None } }), mana_component: false, tap: false, sacrifice: false, exile_self: false, discard: false, life: 0, remove: None, loyalty: None },
        targets: TargetPlan::NONE, program: &[Instruction::GrantKeywords { subject: Subject::Source, keywords: Keywords::LIFELINK }], sorcery: false, once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
