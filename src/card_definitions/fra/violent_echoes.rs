// Direct typed behavior from pinned Oracle and XMage; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 65,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        &[
            Instruction::DamageForExcess(AmountValue::Fixed(6)),
            Instruction::IfNoExcessDamage { skip: 2 },
            Instruction::EnsureJaceToken,
            Instruction::EmpowerJaceExcessDamage,
        ],
    ),
    ..CardRules::EMPTY
};
