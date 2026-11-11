const FILTER: CreatureFilter = CreatureFilter {
    relation: 0,
    other: true,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersBatchYou {
            predicate: EntryPredicate::SubtypeAny(&["Rabbit", "Bat", "Bird", "Mouse"]),
            other: true,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Scry(1)],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::CreatureGroupBonus {
            filters: &[
                CreatureFilter {
                    subtype: Some("Rabbit"),
                    ..FILTER
                },
                CreatureFilter {
                    subtype: Some("Bat"),
                    ..FILTER
                },
                CreatureFilter {
                    subtype: Some("Bird"),
                    ..FILTER
                },
                CreatureFilter {
                    subtype: Some("Mouse"),
                    ..FILTER
                },
            ],
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
    }],
    ..CardRules::EMPTY
};
