// Filter each actual entrant, including a modified source only if it really has the subtype.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[
        StaticAbilityFragment::ControlledSubtypeKeywords {
            subtype: "Mount",
            keywords: Keywords::HASTE,
        },
        StaticAbilityFragment::ControlledSubtypeKeywords {
            subtype: "Vehicle",
            keywords: Keywords::HASTE,
        },
    ],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    static_groups: GROUPS,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::FilteredEntersYou(EntryPredicate::SubtypeAny(&["Mount"])),
            targets: TargetPlan::NONE,
            program: &[Instruction::MarkSaddled(Subject::Entering)],
        },
        TriggeredAbility {
            condition: TriggerCondition::FilteredEntersYou(EntryPredicate::SubtypeAny(&[
                "Vehicle",
            ])),
            targets: TargetPlan::NONE,
            program: &[Instruction::AnimateSubject {
                subject: Subject::Entering,
                spec: AnimationSpec {
                    artifact: true,
                    creature_types: AnimationCreatureTypes::Unchanged,
                    base: None,
                    keywords: Keywords::NONE,
                    duration: AnimationDuration::EndOfTurn,
                },
            }],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
