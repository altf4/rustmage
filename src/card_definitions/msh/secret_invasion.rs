// Freeze the target's copiable values before exile, including tokens/current copies.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds: 1,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    })),
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
            targets: TargetPlan::Single(TargetSpec {
                min: 0,
                max: 1,
                rule: TargetRule::Filtered {
                    base: &TargetRule::Creature,
                    predicate: &TargetPredicate::OtherThanSourceAttachment,
                },
            }),
            program: &[
                Instruction::ExileAndCaptureAttachedCopy,
                Instruction::ApplyCapturedAttachedCopy,
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::Ward),
            targets: TargetPlan::NONE,
            program: &[Instruction::CounterUnlessPay {
                cost: ManaCost {
                    colored: [0; 6],
                    generic: 2,
                },
                subject: CounterSubject::Related,
                exile: false,
                unpaid_skip: 0,
            }],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachmentTrigger(1)],
    }],
    ..CardRules::EMPTY
};
