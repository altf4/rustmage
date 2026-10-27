// Actual exact graveyard-history batch; Food form and retained abilities exist as each card enters.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::ReturnOpponentsBattlefieldGraveyardCardsAsFood],
    ),
    extras: &CardExtras {
        self_granted_activation_templates: &[0],
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 2,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: true,
            sacrifice: true,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GainLife(PlayerSet::Controller, 3)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
