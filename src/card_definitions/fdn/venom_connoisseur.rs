// Direct typed behavior; complete Oracle and pinned executable sources reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OtherCreatureEntersYou,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GrantKeywords {
                subject: Subject::Source,
                keywords: Keywords::DEATHTOUCH,
            },
            Instruction::IfThisAbilityResolutionNumber { number: 2, skip: 1 },
            Instruction::GrantCreatures {
                filter: CreatureFilter {
                    relation: 0,
                    other: false,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                keywords: Keywords::DEATHTOUCH,
            },
        ],
    }],
    ..CardRules::EMPTY
};
