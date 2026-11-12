// Direct typed behavior from the pinned Oracle and XMage sources.
const BONUS: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 2,
        toughness: 2,
    },
    keywords: Keywords::VIGILANCE,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Creature)),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: BONUS,
    }),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Filtered {
            base: &TargetRule::creature_you_control(),
            predicate: &TargetPredicate::OtherThanAuraAttachment,
        }),
        program: &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 2,
                toughness: 2,
            },
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::VIGILANCE,
            },
        ],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::AttachmentStats(BONUS.stats),
            StaticAbilityFragment::AttachmentKeywords(BONUS.keywords),
        ],
    }],
    ..CardRules::EMPTY
};
