pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Ward,
        targets: TargetPlan::NONE,
        program: &[Instruction::CounterUnlessPay {
            cost: generic(1),
            subject: CounterSubject::Related,
            exile: false,
            unpaid_skip: 0,
        }],
    }],
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        guard: ActivationGuard::Always,
        cost: ActivationCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 3,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrowScaled {
            subject: Subject::Source,
            amount: AmountValue::SourceToughness,
            power: 1,
            toughness: 0,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
