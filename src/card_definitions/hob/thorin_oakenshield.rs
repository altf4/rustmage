// Complete printed obligations from the pinned Oracle and full XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::TRAMPLE,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Storied],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::ConditionalWard {
                condition: StatValue::HasEnduringStory,
                filter: CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 17,
                    exclude: 0,
                    subtype: None,
                    basic: false,
                    other: false,
                    cards_only: false,
                },
                ability: 0,
            }],
        },
    ],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::Ward),
        targets: TargetPlan::NONE,
        program: &[Instruction::CounterUnlessPay {
            cost: ManaCost {
                colored: [0; 6],
                generic: 1,
            },
            subject: CounterSubject::Related,
            exile: false,
            unpaid_skip: 0,
        }],
    }],
    ..CardRules::EMPTY
};
