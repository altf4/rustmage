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
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::TapAttached],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::AttachedUntap(
            UntapRestriction::ControllerStep,
        )],
    }],
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        guard: ActivationGuard::Always,
        cost: ActivationCost {
            colored: [0, 2, 0, 0, 0, 0],
            generic: 4,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ShuffleAttachedIntoOwnerLibrary,
            Instruction::ManifestDread,
        ],
        cycling: false,
        exhaust: false,
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
