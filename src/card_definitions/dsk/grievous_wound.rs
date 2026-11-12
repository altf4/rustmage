pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Player {
        opponent_only: false,
    })),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AttachedPlayerDamaged,
        targets: TargetPlan::NONE,
        program: &[Instruction::LoseHalfLifeRoundedUp(PlayerSet::DamagedPlayer)],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::EnchantedPlayerCantGainLife],
    }],
    extras: &CardExtras {
        enchant_player: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
