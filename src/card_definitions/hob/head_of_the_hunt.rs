// The replacement creates a real independently resolving reflexive ability.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::FLASH,
    extras: &CardExtras {
        zone_replacement: Some(ZoneReplacementRule::ExileOpposingCreatureDeath { ability: 0 }),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5412),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
