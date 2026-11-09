// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::DamagePlayersScaled(
            PlayerSet::Opponent,
            AmountValue::Stat(StatValue::Count(CountFilter {
                zone: CountZone::Graveyard,
                relation: 0,
                kind: 0,
                exclude: 1 | 2,
                subtype: None,
                basic: false,
                other: false,
                cards_only: true,
            })),
        )],
    }],
    ..CardRules::EMPTY
};
