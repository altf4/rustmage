pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::ControllerHandSize),
                minimum: 2,
                skip: 2,
            },
            Instruction::May { skip: 1 },
            Instruction::DiscardExactlyReflexiveManaValue {
                amount: 2,
                ability: 0,
            },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::DamagePlayerPermanents {
            players: PlayerSet::Each,
            kinds: 1,
            amount: AmountValue::DiscardedCardManaValue,
        }],
    }],
    ..CardRules::EMPTY
};
