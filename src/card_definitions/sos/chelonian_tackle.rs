// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::creature_you_control()),
            TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 1,
                    relation: 1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 1,
            },
        ]),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 0,
                toughness: 10,
            },
            Instruction::Fight {
                source: 0,
                recipient: 1,
            },
        ],
    ),
    ..CardRules::EMPTY
};
