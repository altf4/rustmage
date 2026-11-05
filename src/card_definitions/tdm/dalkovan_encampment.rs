// Directly authored from the pinned Oracle and complete XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::UnlessControlsEitherSubtype("Swamp", "Mountain"),
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                1, 0, 0, 0, 0, 0,
            ]))],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 2,
                mana_component: true,
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::ScheduleAttackTriggerThisTurn { ability: 0 }],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedTargetEvent,
        targets: TargetPlan::NONE,
        program: &[Instruction::Mobilize {
            token: Card(5255),
            amount: AmountValue::Fixed(2),
        }],
    }],
    ..CardRules::EMPTY
};
