pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::VIGILANCE,
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Fixed([0, 0, 0, 1, 1, 0]),
            quality: &ManaQuality {
                restriction: ManaRestriction::FaceDownSpellOrTurnCreatureFaceUp,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })],
    }],
    ..CardRules::EMPTY
};
