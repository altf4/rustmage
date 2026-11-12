pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GainLife(PlayerSet::Controller, 1),
            Instruction::Surveil(1),
        ],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLYING)],
    }],
    extras: &CardExtras {
        disguise_cost: Some(ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 4,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
