pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTurnedFaceUp,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 16 | 32,
            relation: 1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        program: &[Instruction::Destroy],
    }],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec::fixed(ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 5,
        })),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
