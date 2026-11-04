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
            Instruction::SelectGroup(0),
            Instruction::MultiplyTargetStats { factor: 2 },
            Instruction::Fight {
                source: 0,
                recipient: 1,
            },
        ],
    ),
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 1, 0, 0, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 1, 0, 1, 0, 0],
                generic: 2,
            },
            ManaCost {
                colored: [0, 0, 0, 0, 1, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 0, 0, 1, 1, 0],
                generic: 2,
            },
            ManaCost {
                colored: [0, 1, 0, 0, 1, 0],
                generic: 2,
            },
            ManaCost {
                colored: [0, 1, 0, 1, 1, 0],
                generic: 0,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
