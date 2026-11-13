// Direct typed behavior from the pinned Oracle and XMage sources.
const BONUS: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 1,
        toughness: 0,
    },
    keywords: Keywords::NONE,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::JobSelect {
            token: Card::from_catalogue_id(5272),
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::AttachmentStats(BONUS.stats)],
    }],
    extras: &CardExtras {
        equipment: Some(BONUS),
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        sorcery: true,
        once_per_turn: false,
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::AttachSource],
    }],
    ..CardRules::EMPTY
};
