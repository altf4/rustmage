// Direct typed behavior from complete pinned Oracle and XMage sources.
const STASH: &[Instruction] = &[Instruction::PutCounters {
    subject: Subject::Source,
    kind: CounterKind::Stash,
    amount: 1,
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: STASH,
        },
        TriggeredAbility {
            condition: TriggerCondition::Expend(4),
            targets: TargetPlan::NONE,
            program: STASH,
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 1,
            mana_component: true,
            sacrifice: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::DiscardHandRememberCount(PlayerSet::Controller),
            Instruction::DrawScaled(
                PlayerSet::Controller,
                AmountValue::SourceCounterQuantity(CounterKind::Stash),
            ),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
