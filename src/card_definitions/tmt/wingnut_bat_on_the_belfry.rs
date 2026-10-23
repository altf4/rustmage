// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::OtherCreatureEntersYou,
            targets: TargetPlan::NONE,
            program: &[Instruction::ChooseSourceKeyword(&[
                ("Flying", Keywords::FLYING),
                ("Menace", Keywords::MENACE),
                ("Haste", Keywords::HASTE),
            ])],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::GrowCreatures {
                filter: CreatureFilter {
                    relation: 2,
                    other: true,
                    subtype: None,
                    keyword: Keywords::NONE,
                    attacking: true,
                    tokens: None,
                    artifact: false,
                },
                power: 1,
                toughness: 0,
            }],
        },
    ],
    ..CardRules::EMPTY
};
