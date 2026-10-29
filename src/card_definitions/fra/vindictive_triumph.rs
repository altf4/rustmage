// Complete Oracle and pinned XMage behavior; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::Object {
        zone: TargetZone::Battlefield, kinds: 65, relation: -1, other: false,
        nonland: false, tapped: false, min_mana: 0, max_mana: u32::MAX,
    }), &[Instruction::ExileSmallReturnUntilEndStep]),
    ..CardRules::EMPTY
};
