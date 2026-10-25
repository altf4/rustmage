// Whole printed behavior from the complete frozen Oracle and pinned Java sources.
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Type,
    fragments: &[
        StaticAbilityFragment::SourceGraveyardAllTypes {
            kind: 1,
            minimum: 2,
        },
        StaticAbilityFragment::SourceGraveyardBonus {
            kind: 1,
            minimum: 2,
            stats: Stats {
                power: 2,
                toughness: 2,
            },
        },
    ],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
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
        program: &[Instruction::AddMana(ManaProduction::Choice {
            mask: 31,
            amount: 1,
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
