// Oracle and pinned XMage evidence are recorded in the paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SourceDealtDamageToCreatureSubtype("Spider"),
        targets: TargetPlan::NONE,
        program: &[Instruction::DestroyEventPermanent],
    }],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Graveyard,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            generic: 6,
            mana_component: true,
            exile_self: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5262),
            amount: 2,
            tapped: true,
        }],
    }],
    ..CardRules::EMPTY
};
