// Gated core fixture: ordinary typed obligations, not an admission declaration.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: false,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::ChosenColor)],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        entry_choice: Some(EntryChoiceKind::Color),
        ..CardExtras::EMPTY
    },
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::ChosenModifier(ChosenModifier {
            kind: EntryChoiceKind::Color,
            other: false,
            stats: Stats {
                power: 1,
                toughness: 0,
            },
            source_counter: None,
        })],
    }],
    ..CardRules::EMPTY
};
