// Direct typed behavior; complete pinned Oracle and XMage source reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 0),
    enters: Counters {
        plus_one: 4,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 2, 0],
            generic: 2,
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
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::PutCountersScaled {
            subject: Subject::Source,
            kind: CounterKind::PlusOne,
            amount: AmountValue::SourcePower,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
