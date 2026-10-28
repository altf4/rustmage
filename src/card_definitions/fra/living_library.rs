// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(0, 4),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 65,
            relation: 1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 6,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: true,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        program: &[Instruction::ShuffleTargetIntoLibrary],
    }],
    ..CardRules::EMPTY
};
