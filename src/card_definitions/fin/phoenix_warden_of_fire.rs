const RISING_FLAMES: TriggeredAbility = TriggeredAbility {
    condition: TriggerCondition::Chapter { number: 1 },
    targets: TargetPlan::NONE,
    program: &[Instruction::DamagePlayers(PlayerSet::Opponent, 2)],
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING.union(Keywords::LIFELINK),
    triggers: &[
        RISING_FLAMES,
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            ..RISING_FLAMES
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Grouped {
                    base: &TargetRule::Object {
                        zone: TargetZone::Graveyard,
                        kinds: 1,
                        relation: 0,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    constraint: TargetGroupConstraint::TotalManaValueAtMost(6),
                },
                min: 0,
                max: usize::MAX,
            }),
            program: &[
                Instruction::ReturnToBattlefield,
                Instruction::ExileBattlefieldSourceReturn {
                    transformed: false,
                    under_owner: false,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
