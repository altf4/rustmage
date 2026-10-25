// Direct typed ability-copy behavior from full frozen Oracle and pinned Java.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::NONE,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::OwnTurn,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: false,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 2,
            remove: None,
            loyalty: None,
        },
        targets: one(TargetRule::YourAbilityFromSourceKinds { kinds: 16 }),
        program: &[Instruction::CopyTargetAbility { group: 0 }],
        sorcery: false,
        once_per_turn: true,
    }],
    ..CardRules::EMPTY
};
