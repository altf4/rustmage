// Direct typed behavior from the pinned Oracle and XMage sources.
const OTHERS: CreatureFilter = CreatureFilter {
    relation: 0,
    other: true,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: &[Instruction::RevealTopMoveIfPermanent],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 2, 0,
            ]))],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::GrowCreatures {
                    filter: OTHERS,
                    power: 2,
                    toughness: 2,
                },
                Instruction::GrantCreatures {
                    filter: OTHERS,
                    keywords: Keywords::TRAMPLE,
                },
            ],
        },
    ],
    ..CardRules::EMPTY
};
