// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ManaSpent {
            minimum: 0,
            increment: false,
            event: &TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Noncreature,
                ordinal: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::CreateTokens {
                token: Card(5328),
                amount: 1,
                tapped: false,
            },
            Instruction::IfManaSpent {
                minimum: Some(4),
                maximum: None,
                skip: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 2),
            Instruction::IfManaSpent {
                minimum: Some(8),
                maximum: None,
                skip: 2,
            },
            Instruction::SacrificeSource,
            Instruction::DamagePlayersScaled(PlayerSet::Opponent, AmountValue::ManaSpent),
        ],
    }],
    ..CardRules::EMPTY
};
