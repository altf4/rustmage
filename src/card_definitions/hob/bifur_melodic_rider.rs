// Oracle and official release notes; XMage absence is explicitly pinned in JSON.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::Storied],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::AdditionalTriggers {
                scope: AdditionalTriggerScope::ControlledSubtype("Dwarf"),
                condition: StatValue::HasEnduringStory,
            }],
        },
    ],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[Instruction::PutCounters {
            subject: Subject::Targets,
            kind: CounterKind::PlusOne,
            amount: 1,
        }],
    }],
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [1, 0, 0, 1, 0, 0],
                generic: 4,
            },
            ManaCost {
                colored: [2, 0, 0, 0, 0, 0],
                generic: 4,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
