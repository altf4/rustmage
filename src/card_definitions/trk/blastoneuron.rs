// Oracle and official Star Trek release notes; no pinned XMage class exists.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: 3,
            }),
            program: &[
                Instruction::ExileGraveyardTargetCopyWithNextEndStepSacrifice {
                    keywords: Keywords::HASTE,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::CardsLeaveGraveyard,
            targets: TargetPlan::NONE,
            program: &[Instruction::LoseOpponentGainSame(AmountValue::Fixed(1))],
        },
    ],
    ..CardRules::EMPTY
};
