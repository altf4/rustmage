// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::GrowPlayerCreatures {
                players: PlayerSet::Each,
                power: -2,
                toughness: -2,
            },
            Instruction::IfWaterbendPaid { skip: 1 },
            Instruction::ScheduleTurnTrigger {
                event: TurnCallbackEvent::AnyCreatureDeath,
                ability: 0,
                once: false,
            },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 1)],
    }],
    extras: &CardExtras {
        waterbend_additional: Some(WaterbendAdditional::Optional(4)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
