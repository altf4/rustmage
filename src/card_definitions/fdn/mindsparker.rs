// Direct typed behavior; complete Oracle and pinned XMage reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::FIRST_STRIKE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::Opponent,
            filter: CastFilter::Colors {
                filter: &CastFilter::InstantSorcery,
                mask: 3,
            },
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::DamagePlayers(PlayerSet::CastingPlayer, 2)],
    }],
    ..CardRules::EMPTY
};
