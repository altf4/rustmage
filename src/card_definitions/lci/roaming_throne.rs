// The chosen type is attached to the exact active entry-choice ability.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Ward,
        targets: TargetPlan::NONE,
        program: &[Instruction::CounterUnlessPay {
            cost: ManaCost {
                colored: [0; 6],
                generic: 2,
            },
            subject: CounterSubject::Related,
            exile: false,
            unpaid_skip: 0,
        }],
    }],
    static_groups: &[
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Type,
            fragments: &[StaticAbilityFragment::ChosenCreatureType],
        },
        StaticAbilityGroup {
            line: 3,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AdditionalTriggers {
                scope: AdditionalTriggerScope::OtherControlledCreatureOfChosenType,
                condition: StatValue::Fixed(1),
            }],
        },
    ],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::CreatureType),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
