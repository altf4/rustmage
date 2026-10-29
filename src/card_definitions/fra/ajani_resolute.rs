// Complete pinned Oracle and XMage behavior; gated pending integration.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras { starting_loyalty: 2, ..CardExtras::EMPTY },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LifeGained { first_only: false, own_turn: false },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters { subject: Subject::Source, kind: CounterKind::Loyalty, amount: 1 }],
    }],
    activated: &[
        ActivatedAbility { exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
            cycling: false, cost: ActivationCost { colored: [0; 6], generic: 0, alternatives: &[],
                additional: None, mana_component: false, tap: false, sacrifice: false, exile_self: false,
                discard: false, life: 0, remove: None, loyalty: Some(0) },
            targets: TargetPlan::NONE, program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
            sorcery: true, once_per_turn: false },
        ActivatedAbility { exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
            cycling: false, cost: ActivationCost { colored: [0; 6], generic: 0, alternatives: &[],
                additional: None, mana_component: false, tap: false, sacrifice: false, exile_self: false,
                discard: false, life: 0, remove: None, loyalty: Some(-4) },
            targets: TargetPlan::NONE, program: &[Instruction::CreateTokens { token: Card(5462), amount: 1, tapped: false }],
            sorcery: true, once_per_turn: false },
        ActivatedAbility { exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
            cycling: false, cost: ActivationCost { colored: [0; 6], generic: 0, alternatives: &[],
                additional: None, mana_component: false, tap: false, sacrifice: false, exile_self: false,
                discard: false, life: 0, remove: None, loyalty: Some(-10) },
            targets: TargetPlan::NONE, program: &[Instruction::CreateEmblem(Card(5463))],
            sorcery: true, once_per_turn: false },
    ],
    ..CardRules::EMPTY
};
