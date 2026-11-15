pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 0,
                    relation: 0,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                predicate: &TargetPredicate::Permanent,
            }),
            program: &[Instruction::ReturnToHand],
        },
        TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(&TriggerCondition::CardsLeaveGraveyard),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card::from_catalogue_id(6011),
                amount: 1,
                tapped: false,
            }],
        },
    ],
    ..CardRules::EMPTY
};
