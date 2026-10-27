// Exact two-target all-or-nothing exchange; legality uses current card types.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            min: 2,
            max: 2,
            rule: TargetRule::Grouped {
                base: &TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 0,
                    relation: -1,
                    other: false,
                    nonland: true,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                constraint: TargetGroupConstraint::SharedCardType,
            },
        }),
        &[Instruction::ExchangeControlTargets],
    ),
    ..CardRules::EMPTY
};
