// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ModifyPlayerCreatures {
                players: PlayerSet::Controller,
                power: 2,
                toughness: 0,
                keywords: Keywords::NONE,
            },
            Instruction::ScheduleTurnTrigger {
                event: TurnCallbackEvent::ControlledNontokenCreatureDeath,
                ability: 0,
                once: false,
            },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5323),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
