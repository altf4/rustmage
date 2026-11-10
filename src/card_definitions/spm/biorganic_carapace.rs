const BONUS: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 2,
        toughness: 2,
    },
    keywords: Keywords::NONE,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: one(TargetRule::creature_you_control()),
            program: &[Instruction::AttachSource],
        },
        TriggeredAbility {
            condition: TriggerCondition::Granted(&TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Source,
                combat_only: true,
                opponent_only: false,
                one_or_more: false,
            }),
            targets: TargetPlan::NONE,
            program: &[Instruction::DrawScaled(
                PlayerSet::Controller,
                AmountValue::Stat(StatValue::ControlledModifiedCreatures),
            )],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::AttachSource],
        sorcery: true,
        once_per_turn: false,
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::AttachmentStats(BONUS.stats),
            StaticAbilityFragment::AttachmentTrigger(1),
        ],
    }],
    extras: &CardExtras {
        equipment: Some(BONUS),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
