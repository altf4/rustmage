// Each off-turn cast damages the actual caster.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::Any,
            filter: CastFilter::CasterOffTurn {
                filter: &CastFilter::Any,
            },
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::DamagePlayers(PlayerSet::CastingPlayer, 4)],
    }],
    ..CardRules::EMPTY
};
