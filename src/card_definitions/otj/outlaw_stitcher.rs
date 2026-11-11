// Complete typed behavior from the pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensThenCounters {
            token: Card(5305),
            amount: 1,
            kind: CounterKind::PlusOne,
            counters: AmountValue::Multiple {
                value: &AmountValue::Stat(StatValue::SpellsCastAfterFirstThisTurn),
                factor: 2,
            },
        }],
    }],
    extras: &CardExtras {
        plot_cost: Some(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
