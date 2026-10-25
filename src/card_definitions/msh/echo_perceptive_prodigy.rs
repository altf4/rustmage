// Direct typed ability-copy behavior from full frozen Oracle and pinned Java.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    keywords: Keywords::VIGILANCE,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 1,
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
        targets: one(TargetRule::YourAbilityFromSourceKinds { kinds: 1 }),
        program: &[Instruction::CopyTargetAbility { group: 0 }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
