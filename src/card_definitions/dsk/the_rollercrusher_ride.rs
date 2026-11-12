const DOUBLE_NONCOMBAT_DAMAGE: ReplacementSpec =
    ReplacementSpec::ConditionalDoubleControlledNoncombatDamage {
        condition: StatValue::AtLeast {
            value: &StatValue::CONTROLLER_GRAVEYARD_CARD_TYPES,
            minimum: 4,
        },
    };

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: usize::MAX,
        }),
        program: &[
            Instruction::UpToTargets(AmountValue::ChosenX),
            Instruction::DamageScaled(AmountValue::ChosenX),
        ],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::Replacement(
            0,
            DOUBLE_NONCOMBAT_DAMAGE,
        )],
    }],
    extras: &CardExtras {
        x_symbols: 1,
        replacement_effects: &[DOUBLE_NONCOMBAT_DAMAGE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
