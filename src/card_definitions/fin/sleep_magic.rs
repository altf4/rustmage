// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
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
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::TapAttached],
        },
        TriggeredAbility {
            condition: TriggerCondition::AttachedCreatureDamaged,
            targets: TargetPlan::NONE,
            program: &[Instruction::SacrificeSource],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AttachedUntap(
            UntapRestriction::ControllerStep,
        )],
    }],
    ..CardRules::EMPTY
};
