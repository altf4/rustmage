pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::creature_you_control()),
        &[
            Instruction::CreaturePowerDamageBatch {
                source: 0,
                recipients: CreaturePowerDamageRecipients::OtherCreatures,
            },
            Instruction::ScheduleTurnTrigger {
                event: TurnCallbackEvent::ControlledCreatureDeath,
                ability: 0,
                once: false,
            },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileTopPlay {
            amount: 1,
            next_turn: true,
        }],
    }],
    ..CardRules::EMPTY
};
