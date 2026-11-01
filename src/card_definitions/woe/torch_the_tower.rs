// Actual, unprevented damage recipients receive the independent death replacement.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 65,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[
            Instruction::ExileDamagedIfDiesThisTurn,
            Instruction::IfBargained { skip: 3 },
            Instruction::Damage(3),
            Instruction::Scry(1),
            Instruction::Skip(1),
            Instruction::Damage(2),
        ],
    ),
    extras: &CardExtras {
        bargain_reduction: Some(0),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
