// Complete printed obligations from pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    entry: EntryTapped::UnlessControlsSubtype("Equipment"),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 4,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5407),
            amount: 1,
            tapped: false,
        }],
        sorcery: true,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        activation_subtype_discounts: &[(0, "Equipment")],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
