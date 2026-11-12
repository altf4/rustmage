// Directly authored from the paired Oracle and pinned XMage sources.
const ABILITIES: Keywords = Keywords::TRAMPLE.union(Keywords::HASTE);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: ABILITIES,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(ABILITIES)],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::StatAtLeast {
            value: StatValue::CONTROLLER_GRAVEYARD_CARD_TYPES,
            minimum: 4,
        },
        origin: ActivationOrigin::Graveyard,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 2, 0],
            generic: 2,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnSourceToBattlefieldWithCounter {
            tapped: false,
            counter: CounterKind::Finality,
        }],
        sorcery: true,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        uncounterable: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
