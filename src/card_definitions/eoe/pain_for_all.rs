// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::creature_you_control())),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: 0,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Any,
                predicate: &TargetPredicate::OtherThanAuraAttachment,
            }),
            program: &[Instruction::AttachedCreaturePowerDamage],
        },
        TriggeredAbility {
            condition: TriggerCondition::AttachedCreatureDamaged,
            targets: TargetPlan::NONE,
            program: &[Instruction::EventObjectDamageReceivedPlayers(
                PlayerSet::Opponent,
            )],
        },
    ],
    ..CardRules::EMPTY
};
