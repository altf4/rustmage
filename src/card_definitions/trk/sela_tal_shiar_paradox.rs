// Complete Oracle behavior, checked against pinned XMage or official release notes.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 0,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 1,
            }),
            program: &[Instruction::Exile],
        },
        TriggeredAbility {
            condition: TriggerCondition::CardsLeaveGraveyard,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card(6012),
                amount: 1,
                tapped: true,
            }],
        },
    ],
    ..CardRules::EMPTY
};
