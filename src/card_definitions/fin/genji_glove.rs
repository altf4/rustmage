// The combat packet identifies the original attacker even if the Glove moves.
const BONUS: EquipmentSpec = EquipmentSpec {
    stats: Stats {
        power: 0,
        toughness: 0,
    },
    keywords: Keywords::DOUBLE_STRIKE,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachmentKeywords(BONUS.keywords)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::FirstCombatPhase,
            event: &TriggerCondition::EquippedAttacks,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::UntapEventObject, Instruction::AdditionalCombat],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            generic: 3,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::AttachSource],
        sorcery: true,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        equipment: Some(BONUS),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
