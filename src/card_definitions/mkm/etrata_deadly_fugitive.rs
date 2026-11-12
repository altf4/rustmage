pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    keywords: Keywords::DEATHTOUCH,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledFaceDownCreatureActivation(
            0,
        )],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 1, 1, 0, 0, 0],
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::TurnSourceFaceUpOrExile,
            Instruction::CastRememberedExileFree,
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::ControlledSubtypes(&["Assassin"]),
            combat_only: true,
            opponent_only: true,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CloakLibraryTop(PlayerSet::DamagedPlayer)],
    }],
    ..CardRules::EMPTY
};
