// Direct Oracle behavior and pinned XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::PutCountersScaled {
            subject: Subject::Targets,
            kind: CounterKind::MinusOne,
            amount: AmountValue::Stat(StatValue::GreatestManaValue(CountFilter {
                zone: CountZone::Graveyard,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: true,
            })),
        }],
    }],
    ..CardRules::EMPTY
};
