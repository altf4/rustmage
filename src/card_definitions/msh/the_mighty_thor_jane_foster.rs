// Complete frozen Oracle and pinned XMage behavior, with exact tapped owner returns.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Filtered {
                    base: &TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 17,
                        relation: -1,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    predicate: &TargetPredicate::Nontoken,
                },
                min: 0,
                max: 1,
            }),
            program: &[Instruction::BlinkImmediately {
                counter: None,
                tapped: true,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SubtypeEntersYou("Equipment"),
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ],
    ..CardRules::EMPTY
};
