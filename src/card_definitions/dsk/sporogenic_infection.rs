pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Creature)),
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
            targets: one(TargetRule::Player {
                opponent_only: false,
            }),
            program: &[Instruction::ChooseObjects {
                kind: EffectChoiceKind::SacrificeExceptSourceAttachment,
                players: PlayerSet::Targeted,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: None,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::AttachedCreatureDamaged,
            targets: TargetPlan::NONE,
            program: &[Instruction::DestroyEventPermanent],
        },
    ],
    ..CardRules::EMPTY
};
