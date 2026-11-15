// Oracle and pinned XMage reviewed. The graveyard mode can target either graveyard.
const GRAVE_CARD: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 0,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const SMALL_ARTIFACT_OR_ENCHANTMENT: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 16 | 32,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: 3,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Alter the Past — exile a graveyard card and gain two life",
            targets: one(GRAVE_CARD),
            program: &[Instruction::Exile, Instruction::GainLife(PlayerSet::Controller, 2)],
        },
        SpellMode {
            text: "Obliterate the Transwarp Network — destroy a small artifact or enchantment",
            targets: one(SMALL_ARTIFACT_OR_ENCHANTMENT),
            program: &[Instruction::Destroy],
        },
    ]),
    ..CardRules::EMPTY
};
