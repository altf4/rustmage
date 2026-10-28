// Direct Oracle behavior and pinned XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CounterPlacedByYou {
            kind: CounterKind::Loyalty,
            filter: CountFilter {
                zone: CountZone::Battlefield,
                relation: -1,
                kind: 64,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 6,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::EnsureJaceToken, Instruction::EmpowerJace(2)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
