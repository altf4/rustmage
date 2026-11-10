const YOUR_GRAVEYARD: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: true,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec {
                rule: TargetRule::Filtered {
                    base: &YOUR_GRAVEYARD,
                    predicate: &TargetPredicate::Permanent,
                },
                min: 0,
                max: 2,
            },
            TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 2,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 2,
            },
        ]),
        &[
            Instruction::SelectGroup(0),
            Instruction::ReturnToHand,
            Instruction::SelectGroup(1),
            Instruction::ReturnToBattlefieldTapped,
        ],
    ),
    ..CardRules::EMPTY
};
