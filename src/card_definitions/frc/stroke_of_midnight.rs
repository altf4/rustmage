// Direct typed behavior from the complete paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(one(TargetRule::Object { zone: TargetZone::Battlefield, kinds: 0, relation: -1, other: false, nonland: true, tapped: false, min_mana: 0, max_mana: u32::MAX }),
        &[Instruction::DestroyAndCreateForController { token: Card(5254), amount: 1 }]),
    ..CardRules::EMPTY
};
