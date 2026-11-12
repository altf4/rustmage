const BONUS: Stats = Stats {
    power: 2,
    toughness: 0,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::AttachmentStats(BONUS)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::EquippedAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::SacrificeExceptAttackingAttachment,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: true,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: Some(1),
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
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
    extras: &CardExtras {
        equipment: Some(EquipmentSpec {
            stats: BONUS,
            keywords: Keywords::NONE,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
