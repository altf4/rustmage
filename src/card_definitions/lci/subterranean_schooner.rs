pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: one(TargetRule::Filtered {
            base: &TargetRule::Creature,
            predicate: &TargetPredicate::CrewedSourceThisTurn,
        }),
        program: &[Instruction::Explore(Subject::Targets)],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost::ZERO,
        targets: TargetPlan::NONE,
        program: &[Instruction::Crew { minimum: 1 }],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 3,
            toughness: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
