// Oracle and official release notes reviewed; no pinned XMage class exists.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfCast,
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyCastSpell {
                prior_spells: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::When {
                guard: StepGuard::OtherControlledCreatureWithAttachedSubtype("Communicator"),
                event: &TriggerCondition::GrantedAttacks,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::GrantKeywords {
                subject: Subject::Source,
                keywords: Keywords::CANT_BE_BLOCKED,
            }],
        },
    ],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::AttachSource],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        sorcery: true,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachmentTrigger(1)],
    }],
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
