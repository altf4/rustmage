// Complete typed Oracle behavior; see paired source provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Surveil 2, then draw a card.",
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Surveil(2),
                Instruction::Draw(PlayerSet::Controller, 1),
            ],
        },
        SpellMode {
            text: "Deal 1 damage to each of one or two targets.",
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Any,
                min: 1,
                max: 2,
            }),
            program: &[Instruction::Damage(1)],
        },
        SpellMode {
            text: "Return target nonland permanent to its owner's hand.",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: -1,
                other: false,
                nonland: true,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::ReturnToHand],
        },
    ]),
    ..CardRules::EMPTY
};
