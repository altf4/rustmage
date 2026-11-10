const AURA: AuraSpec = AuraSpec {
    kinds: 1,
    relation: -1,
    bonus: EquipmentSpec {
        stats: Stats {
            power: -5,
            toughness: 0,
        },
        keywords: Keywords::NONE,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 1,
    first_layer: AbilityLayer::Modify,
    fragments: &[StaticAbilityFragment::AttachmentStats(AURA.bonus.stats)],
}];
const DESTROY: &[Instruction] = &[Instruction::DestroyEventPermanent];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Creature)),
    aura: Some(AURA),
    static_groups: GROUPS,
    extras: &CardExtras {
        aura: Some(AURA),
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::AttachedCreatureBecomesTapped,
            targets: TargetPlan::NONE,
            program: DESTROY,
        },
        TriggeredAbility {
            condition: TriggerCondition::AttachedCreatureDamaged,
            targets: TargetPlan::NONE,
            program: DESTROY,
        },
    ],
    ..CardRules::EMPTY
};
