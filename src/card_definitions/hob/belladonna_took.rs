// Full Oracle and XMage: one trigger per token; outcomes use resolution ordinals.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersYou(EntryPredicate::Token),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::IfThisAbilityResolutionNumber { number: 1, skip: 1 },
            Instruction::GainLife(PlayerSet::Controller, 1),
            Instruction::IfThisAbilityResolutionNumber { number: 2, skip: 1 },
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::IfThisAbilityResolutionNumber { number: 3, skip: 1 },
            Instruction::PutCreatureCounters {
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
