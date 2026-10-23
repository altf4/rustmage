pub(in crate::cards) const RULES: CardRules = CardRules {
 effect: Effect::Creature(1, 1),
 triggers: &[TriggeredAbility { condition: TriggerCondition::SelfEnters, targets: TargetPlan::NONE, program: &[Instruction::CreateTokens { token: Card(5273), amount: 1, tapped: false }] }],
 activated: &[ActivatedAbility { exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield, cycling: false,
 cost: ActivationCost { colored: [0; 6], generic: 2, alternatives: &[], additional: None, mana_component: true, tap: true, sacrifice: false, exile_self: false, discard: false, life: 0, remove: None, loyalty: None },
 targets: TargetPlan::NONE, program: &[Instruction::PutCreatureCounters { filter: CreatureFilter { relation: 0, other: false, subtype: None, keyword: Keywords::NONE, attacking: false, tokens: Some(true), artifact: false }, kind: CounterKind::PlusOne, amount: 1 }, Instruction::GrantCreatures { filter: CreatureFilter { relation: 0, other: false, subtype: None, keyword: Keywords::NONE, attacking: false, tokens: Some(true), artifact: false }, keywords: Keywords::TRAMPLE }], sorcery: false, once_per_turn: false }],
 ..CardRules::EMPTY
};
