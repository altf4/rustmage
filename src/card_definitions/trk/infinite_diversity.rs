// Oracle and official Star Trek release notes; pinned XMage class is absent.
const OWN_INSTANT_OR_SORCERY: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 12,
    relation: 0,
    other: false,
    nonland: true,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(OWN_INSTANT_OR_SORCERY),
        &[
            Instruction::ExileTargetCopyCastFreeRepeated(
                StatValue::DistinctControlledCreatureTypesExcept("Borg"),
            ),
            Instruction::ExileResolvingSpell,
        ],
    ),
    ..CardRules::EMPTY
};
