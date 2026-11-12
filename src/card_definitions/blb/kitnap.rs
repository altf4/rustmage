const TARGET: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Creature));

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(TARGET),
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
    static_groups: &[StaticAbilityGroup {
        line: 3,
        first_layer: AbilityLayer::Control,
        fragments: &[StaticAbilityFragment::ControlAttached],
    }],
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfGiftEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::GiveGift],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::TapAttached,
                Instruction::IfGift {
                    promised: false,
                    skip: 1,
                },
                Instruction::PutCounters {
                    subject: Subject::Attached,
                    kind: CounterKind::Stun,
                    amount: 3,
                },
            ],
        },
    ],
    extras: &CardExtras {
        gift: Some(GiftSpec {
            kind: GiftKind::Card,
            token: None,
            promised_targets: TARGET,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
