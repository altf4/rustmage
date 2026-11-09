pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::May { skip: 2 },
            Instruction::DiscardHand(PlayerSet::Controller),
            Instruction::DrawScaled(
                PlayerSet::Controller,
                AmountValue::Devotion(ChoiceColor::Red),
            ),
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 4,
            colored: [0, 0, 0, 2, 0, 0],
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileBattlefieldSourceReturn {
            transformed: true,
            under_owner: true,
        }],
        sorcery: true,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        transforming_double_face: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
