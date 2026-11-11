const CREATURE: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 1,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const PERMANENT: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Object {
        zone: TargetZone::Graveyard,
        kinds: 0,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    },
    predicate: &TargetPredicate::Permanent,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec {
                rule: CREATURE,
                min: 0,
                max: 1,
            },
            TargetSpec {
                rule: PERMANENT,
                min: 0,
                max: 1,
            },
        ]),
        &[
            Instruction::ReturnToBattlefield,
            Instruction::SelectGroup(1),
            Instruction::ReturnToHand,
        ],
    ),
    ..CardRules::EMPTY
};
