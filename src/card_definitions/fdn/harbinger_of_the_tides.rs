// Direct typed behavior from the complete paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Object { zone: TargetZone::Battlefield, kinds: 1, relation: 1, other: false, nonland: false, tapped: true, min_mana: 0, max_mana: u32::MAX }),
        program: &[Instruction::May { skip: 1 }, Instruction::ReturnToHand],
    }],
    extras: &CardExtras { flash_surcharge: Some(generic(2)), ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
