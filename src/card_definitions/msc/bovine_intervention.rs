// Typed Oracle behavior; the provenance record pins the complete XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 17,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        &[Instruction::DestroyAndCreateForController {
            token: Card(5479),
            amount: 1,
        }],
    ),
    ..CardRules::EMPTY
};
