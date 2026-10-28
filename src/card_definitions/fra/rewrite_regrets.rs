// Direct Oracle behavior and pinned XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 65,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: 6,
        }),
        &[
            Instruction::ReturnToBattlefield,
            Instruction::EnsureJaceToken,
            Instruction::EmpowerJace(2),
        ],
    ),
    ..CardRules::EMPTY
};
