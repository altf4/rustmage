// Oracle and pinned XMage reviewed. The modes have distinct graveyard ownership rules.
const ANY_GRAVE_CARD: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 0,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const OWN_GRAVE_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 1,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Plain, Simple Garak — exile a graveyard card and draw",
            targets: one(ANY_GRAVE_CARD),
            program: &[
                Instruction::Exile,
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
        },
        SpellMode {
            text: "Of the Obsidian Order — return your creature card to hand",
            targets: one(OWN_GRAVE_CREATURE),
            program: &[Instruction::ReturnToHand],
        },
    ]),
    ..CardRules::EMPTY
};
