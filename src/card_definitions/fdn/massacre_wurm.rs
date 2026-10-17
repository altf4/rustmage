// Opponent death qualification uses pre-departure characteristics, including simultaneous deaths.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 5),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::GrowCreatures {
                filter: CreatureFilter {
                    relation: 1,
                    other: false,
                    subtype: None,
                    keyword: Keywords(0),
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                power: -2,
                toughness: -2,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::CreatureDies {
                relation: CastRelation::Opponent,
                other: false,
                nontoken: false,
                subtype: None,
            },
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::LoseLife(PlayerSet::Opponent, 2)],
        },
    ],
    ..CardRules::EMPTY
};
