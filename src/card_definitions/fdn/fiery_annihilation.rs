// Direct typed behavior; complete Oracle and pinned executable sources reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Creature),
            TargetSpec {
                rule: TargetRule::Grouped {
                    base: &TargetRule::Filtered {
                        base: &TargetRule::Object {
                            zone: TargetZone::Battlefield,
                            kinds: 0,
                            relation: -1,
                            other: false,
                            nonland: false,
                            tapped: false,
                            min_mana: 0,
                            max_mana: u32::MAX,
                        },
                        predicate: &TargetPredicate::Subtype("Equipment"),
                    },
                    constraint: TargetGroupConstraint::AttachedToGroup(0),
                },
                min: 0,
                max: 1,
            },
        ]),
        &[
            Instruction::Damage(5),
            Instruction::SelectGroup(1),
            Instruction::Exile,
            Instruction::SelectGroup(0),
            Instruction::ExileIfDiesThisTurn,
        ],
    ),
    ..CardRules::EMPTY
};
