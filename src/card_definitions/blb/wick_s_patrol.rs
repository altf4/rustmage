const GRAVEYARD: CountFilter = CountFilter {
    zone: CountZone::Graveyard,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: true,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::MillThenReflexive {
                amount: 3,
                ability: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::GrowScaled {
                subject: Subject::Targets,
                amount: AmountValue::Stat(StatValue::GreatestManaValue(GRAVEYARD)),
                power: -1,
                toughness: -1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
