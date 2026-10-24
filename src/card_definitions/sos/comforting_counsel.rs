// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::LifeGained {
            first_only: false,
            own_turn: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::Growth,
            amount: 1,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::SourceCounterBonus {
            kind: CounterKind::Growth,
            minimum: 5,
            filter: CreatureFilter {
                relation: 0,
                other: false,
                subtype: None,
                keyword: Keywords::NONE,
                attacking: false,
                tokens: None,
                artifact: false,
            },
            stats: Stats {
                power: 3,
                toughness: 3,
            },
        }],
    }],
    ..CardRules::EMPTY
};
