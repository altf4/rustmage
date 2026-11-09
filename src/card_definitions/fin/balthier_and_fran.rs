pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: Keywords::REACH,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::ControlledSubtypeKeywords {
                subtype: "Vehicle",
                keywords: Keywords::REACH.union(Keywords::VIGILANCE),
            },
            StaticAbilityFragment::ControlledCreatureScaling {
                subtype: Some("Vehicle"),
                value: StatValue::Fixed(1),
                stats: Stats {
                    power: 1,
                    toughness: 1,
                },
            },
        ],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::FirstCombatPhase,
            event: &TriggerCondition::CrewedVehicleAttacks,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [0, 0, 0, 1, 1, 0],
                    generic: 1,
                },
                skip: 1,
            },
            Instruction::AdditionalCombat,
        ],
    }],
    ..CardRules::EMPTY
};
