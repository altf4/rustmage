const fn optional_permanent(relation: i8) -> TargetSpec {
    TargetSpec {
        rule: TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1 | 16,
            relation,
            other: true,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        },
        min: 0,
        max: 1,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Groups(&[optional_permanent(0), optional_permanent(1)]),
            program: &[Instruction::MakeTargetsTreasureWhileSourceRemains],
        },
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterUnlessPay {
                cost: generic(1),
                subject: CounterSubject::Related,
                exile: false,
                unpaid_skip: 0,
            }],
        },
    ],
    ..CardRules::EMPTY
};
