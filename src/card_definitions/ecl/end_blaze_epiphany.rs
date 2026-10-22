// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[
            Instruction::DamageScaled(AmountValue::ChosenX),
            Instruction::DelayedDeathThisTurn { ability: 0 },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileTopChoosePlayScaled {
            amount: AmountValue::DeathEventPower,
            next_turn: true,
        }],
    }],
    extras: &CardExtras {
        x_symbols: 1,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
