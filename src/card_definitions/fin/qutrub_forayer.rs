// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Destroy target creature that was dealt damage this turn.",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                    base: &TargetRule::Creature,
                    predicate: &TargetPredicate::Damaged,
                })),
                program: &[Instruction::Destroy],
            },
            SpellMode {
                text: "Exile up to two target cards from a single graveyard.",
                targets: TargetPlan::Single(TargetSpec {
                    rule: TargetRule::Grouped {
                        base: &TargetRule::Object {
                            zone: TargetZone::Graveyard,
                            kinds: 0,
                            relation: -1,
                            other: false,
                            nonland: false,
                            tapped: false,
                            min_mana: 0,
                            max_mana: u32::MAX,
                        },
                        constraint: TargetGroupConstraint::SameGraveyard,
                    },
                    min: 0,
                    max: 2,
                }),
                program: &[Instruction::Exile],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
