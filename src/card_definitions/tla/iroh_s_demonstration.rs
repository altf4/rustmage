// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Deal 1 damage to each opposing creature",
            targets: TargetPlan::NONE,
            program: &[Instruction::DamagePlayerPermanents {
                players: PlayerSet::Opponent,
                kinds: 1,
                amount: AmountValue::Fixed(1),
            }],
        },
        SpellMode {
            text: "Deal 4 damage to target creature",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::Damage(4)],
        },
    ]),
    ..CardRules::EMPTY
};
