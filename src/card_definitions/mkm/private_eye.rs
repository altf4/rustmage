pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::CreatureGroupBonus {
            filters: &[CreatureFilter {
                relation: 0,
                other: true,
                subtype: Some("Detective"),
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            }],
            stats: Stats {
                power: 1,
                toughness: 1,
            },
        }],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CardDrawn {
            relation: CastRelation::You,
            ordinal: Some(2),
        },
        targets: one(TargetRule::Filtered {
            base: &object(0),
            predicate: &TargetPredicate::Subtype("Detective"),
        }),
        program: &[Instruction::GrantKeywords {
            subject: Subject::Targets,
            keywords: Keywords::CANT_BE_BLOCKED,
        }],
    }],
    ..CardRules::EMPTY
};
