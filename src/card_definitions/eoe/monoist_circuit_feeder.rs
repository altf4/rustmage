const fn creature(relation: i8) -> TargetSpec {
    TargetSpec::one(TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 1,
        relation,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    })
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Groups(&[creature(0), creature(1)]),
        program: &[
            Instruction::RememberCount(AmountValue::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 16,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            }))),
            Instruction::GrowScaled {
                subject: Subject::Targets,
                amount: AmountValue::RememberedCount,
                power: 1,
                toughness: 0,
            },
            Instruction::SelectGroup(1),
            Instruction::GrowScaled {
                subject: Subject::Targets,
                amount: AmountValue::RememberedCount,
                power: 0,
                toughness: -1,
            },
        ],
    }],
    ..CardRules::EMPTY
};
