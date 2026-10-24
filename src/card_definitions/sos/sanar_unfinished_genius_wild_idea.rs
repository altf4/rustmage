// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0,4),
    extras: &CardExtras { prepared_spell: Some(Card(5377)), enters_prepared: true, ..CardExtras::EMPTY },
    activated: &[ActivatedAbility { exhaust: false, guard: ActivationGuard::CastInstantSorceryThisTurn, origin: ActivationOrigin::Battlefield, cycling: false,
        cost: ActivationCost { colored: [0;6], generic: 0, alternatives: &[], additional: None, mana_component: false, tap: true, sacrifice: false, exile_self: false, discard: false, life: 0, remove: None, loyalty: None }, targets: TargetPlan::NONE, program: &[Instruction::CreateTokens { token: Card::TreasureToken, amount: 1, tapped: false }], sorcery: false, once_per_turn: false }],
    ..CardRules::EMPTY
};
