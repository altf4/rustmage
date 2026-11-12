const CREATURE_CARD: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 1,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec {
            rule: TargetRule::Grouped {
                base: &CREATURE_CARD,
                constraint: TargetGroupConstraint::SameGraveyard,
            },
            min: 0,
            max: 2,
        }),
        &[Instruction::ReturnGraveyardTargetsWithHasteAndDelayedSacrifice],
    ),
    extras: &CardExtras {
        split_parent: Some(Card::from_catalogue_id(3287)),
        cost_alternatives: &[
            ManaCost {
                colored: [0, 0, 1, 1, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [0, 0, 0, 2, 0, 0],
                generic: 4,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
