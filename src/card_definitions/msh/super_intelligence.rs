// Complete Aura: freeze qualification at upkeep; the upkeep player remains active through resolution.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(object(1))),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            guard: StepGuard::AttachedControllerTurn,
            event: &TriggerCondition::AtStep {
                step: TriggerStep::Upkeep,
                relation: CastRelation::Any,
                guard: StepGuard::Always,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Active, 1)],
    }],
    ..CardRules::EMPTY
};
