pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 7),
    keywords: Keywords::TRAMPLE.union(Keywords::HASTE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::AddCombatRed(4)],
    }],
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::ExpiringManaBecomesRed],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::SourceConditionalKeywords {
                condition: StatValue::AtLeast {
                    value: &StatValue::UnspentMana,
                    minimum: 6,
                },
                keywords: Keywords::FLYING.union(Keywords::INDESTRUCTIBLE),
            }],
        },
    ],
    ..CardRules::EMPTY
};
