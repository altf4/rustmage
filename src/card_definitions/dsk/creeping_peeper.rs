pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        guard: ActivationGuard::Always,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Fixed([0, 1, 0, 0, 0, 0]),
            quality: &ManaQuality {
                restriction: ManaRestriction::EnchantmentSpellOrUnlockDoorOrFaceUp,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })],
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
