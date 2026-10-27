pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Storied],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ConditionalCreatureModifier {
                condition: StatValue::HasEnduringStory,
                modifier: StaticModifier {
                    own_turn: false,
                    requires_life_gain: false,
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
                        power: 1,
                        toughness: 1,
                    },
                },
            }],
        },
    ],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentEntersYou {
            kinds: 0,
            nontoken: true,
            subtype: Some("Dwarf"),
            include_source: true,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5407),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
