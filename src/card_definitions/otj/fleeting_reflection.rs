const YOUR_CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 1,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(YOUR_CREATURE),
            TargetSpec {
                rule: TargetRule::Grouped {
                    base: &TargetRule::Creature,
                    constraint: TargetGroupConstraint::OtherThanGroup(0),
                },
                min: 0,
                max: 1,
            },
        ]),
        &[
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::HEXPROOF,
            },
            Instruction::Untap,
            Instruction::BecomeCopy(InPlaceCopySpec {
                recipient_group: Some(0),
                blueprint_group: 1,
                overrides: crate::engine::CopyOverrides {
                    name: None,
                    legendary: None,
                    kinds: None,
                    types: None,
                    keywords: Keywords::NONE,
                },
                add_kinds: 0,
                base: None,
                next_turn: false,
            }),
        ],
    ),
    ..CardRules::EMPTY
};
