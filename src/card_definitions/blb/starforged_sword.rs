const EQUIPPED_BONUS: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 3,
        toughness: 3,
    },
    keywords: Keywords::NONE,
};
const EQUIP_TARGET: TargetPlan =
    TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control()));

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfGiftEnters,
            targets: TargetPlan::NONE,
            program: &[Instruction::GiveGift],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfGiftEnters,
            targets: EQUIP_TARGET,
            program: &[Instruction::AttachSource],
        },
    ],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            generic: 3,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: EQUIP_TARGET,
        program: &[Instruction::AttachSource],
        cycling: false,
        exhaust: false,
        sorcery: true,
        once_per_turn: false,
    }],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::AttachmentStats(EQUIPPED_BONUS.stats),
            StaticAbilityFragment::RemoveAttachedKeywords(Keywords::FLYING),
        ],
    }],
    extras: &CardExtras {
        equipment: Some(EQUIPPED_BONUS),
        gift: Some(GiftSpec {
            kind: GiftKind::TappedFish,
            token: Some(Card(5317)),
            promised_targets: TargetPlan::NONE,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
