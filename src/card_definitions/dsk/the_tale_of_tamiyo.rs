// Directly authored from the paired Oracle and pinned XMage sources.
const MILL_CHAPTER: &[Instruction] = &[
    Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(2)),
    Instruction::DrawAndRepeatMillingIfSharedType,
];

const fn own_graveyard(kinds: u8) -> TargetRule {
    TargetRule::Object {
        zone: TargetZone::Graveyard,
        kinds,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: MILL_CHAPTER,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: MILL_CHAPTER,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: MILL_CHAPTER,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 4 },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Union(&[
                    own_graveyard(4 | 8),
                    TargetRule::Filtered {
                        base: &own_graveyard(64),
                        predicate: &TargetPredicate::Subtype("Tamiyo"),
                    },
                ]),
                min: 0,
                max: usize::MAX,
            }),
            program: &[Instruction::ExileTargetsCopyCastPaid],
        },
    ],
    ..CardRules::EMPTY
};
