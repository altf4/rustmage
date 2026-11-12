const GRAVE_CARD: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 0,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec {
                rule: TargetRule::Grouped {
                    base: &GRAVE_CARD,
                    constraint: TargetGroupConstraint::SameGraveyard,
                },
                min: 0,
                max: 2,
            },
            TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            }),
        ]),
        &[
            Instruction::Exile,
            Instruction::SelectGroup(1),
            Instruction::GainLife(PlayerSet::Targeted, 2),
            Instruction::Draw(PlayerSet::Targeted, 1),
        ],
    ),
    extras: &CardExtras {
        split_parent: Some(Card::from_catalogue_id(692)),
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 1,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
