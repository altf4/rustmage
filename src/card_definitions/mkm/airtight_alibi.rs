const BONUS: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 2,
        toughness: 2,
    },
    keywords: Keywords::NONE,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
    keywords: Keywords::FLASH,
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: BONUS,
    }),
    static_groups: &[StaticAbilityGroup {
        line: 3,
        first_layer: AbilityLayer::Modify,
        fragments: &[
            StaticAbilityFragment::AttachmentStats(BONUS.stats),
            StaticAbilityFragment::AttachedSuspectProhibited,
        ],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::UntapAttached,
            Instruction::GrantKeywords {
                subject: Subject::Attached,
                keywords: Keywords::HEXPROOF,
            },
            Instruction::Unsuspect {
                subject: Subject::Attached,
            },
        ],
    }],
    ..CardRules::EMPTY
};
