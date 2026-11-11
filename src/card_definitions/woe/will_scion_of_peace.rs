// Direct typed behavior from the pinned Oracle and XMage sources.
const WHITE_AND_BLUE: u8 = 0b00011;

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    keywords: Keywords::VIGILANCE,
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: true,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ReduceColoredSpellCostsThisTurn {
            colors: WHITE_AND_BLUE,
            amount: AmountValue::LifeGainedThisTurn,
        }],
    }],
    ..CardRules::EMPTY
};
