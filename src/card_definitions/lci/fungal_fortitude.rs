const BONUS: Stats = Stats {
    power: 2,
    toughness: 0,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Creature)),
    keywords: Keywords::FLASH,
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: BONUS,
            keywords: Keywords::NONE,
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::AttachmentStats(BONUS)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttachedCreatureDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnEventCardToBattlefieldTapped],
    }],
    ..CardRules::EMPTY
};
