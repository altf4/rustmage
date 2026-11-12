const fn controlled_other(subtype: &'static str) -> CountFilter {
    CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 0,
        exclude: 0,
        subtype: Some(subtype),
        basic: false,
        other: true,
        cards_only: false,
    }
}
const ANIMALS: AmountValue = AmountValue::Stat(StatValue::CountUnion(&[
    controlled_other("Squirrel"),
    controlled_other("Bat"),
    controlled_other("Lizard"),
    controlled_other("Rat"),
]));

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    keywords: Keywords::MENACE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::LoseLifeScaled(PlayerSet::Opponent, ANIMALS),
            Instruction::GainLifeScaled(PlayerSet::Controller, ANIMALS),
        ],
    }],
    ..CardRules::EMPTY
};
