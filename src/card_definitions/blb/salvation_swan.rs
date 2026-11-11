pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FLASH.union(Keywords::FLYING),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersYou(EntryPredicate::SourceOrSubtypeAny(&[
            "Bird",
        ])),
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Filtered {
                base: &TargetRule::creature_you_control(),
                predicate: &TargetPredicate::WithoutKeyword(Keywords::FLYING),
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::BlinkUntilEndStepWithCounter {
            counter: CounterKind::Flying,
            actual_exile_only: true,
        }],
    }],
    ..CardRules::EMPTY
};
