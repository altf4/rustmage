const REANIMATION_TARGET: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Object {
        zone: TargetZone::Graveyard,
        kinds: 0,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: 3,
    },
    predicate: &TargetPredicate::Permanent,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    keywords: Keywords::FLASH,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersIfCastByController,
            targets: one(REANIMATION_TARGET),
            program: &[Instruction::ReturnToBattlefieldTapped],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfBecomesTapped,
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileTopPlay {
                amount: 2,
                next_turn: false,
            }],
        },
    ],
    ..CardRules::EMPTY
};
