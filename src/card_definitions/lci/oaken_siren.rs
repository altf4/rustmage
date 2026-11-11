// Direct typed behavior from the paired Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    keywords: Keywords::FLYING.union(Keywords::VIGILANCE),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Fixed([0, 1, 0, 0, 0, 0]),
            quality: &ManaQuality {
                restriction: ManaRestriction::ArtifactSpellOrArtifactActivation,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
