// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Player {
            opponent_only: true,
        }),
        program: &[Instruction::SetPlayerCreatureCharacteristics {
            players: PlayerSet::Targeted,
            additional_types: &["Coward"],
            power: 1,
            toughness: 1,
            remove_abilities: true,
        }],
    }],
    ..CardRules::EMPTY
};
