// Unwind History prepared face; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::Object {
        zone: TargetZone::Battlefield, kinds: 1, relation: 1, other: false,
        nonland: false, tapped: false, min_mana: 0, max_mana: 3,
    }), &[Instruction::ReturnToHand, Instruction::Surveil(1)]),
    extras: &CardExtras { prepared_identity: Some("@prepare/diviner_of_victory_unwind_history"), ..CardExtras::EMPTY },
    ..CardRules::EMPTY
};
