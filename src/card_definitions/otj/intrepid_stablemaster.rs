const GREEN: ManaProduction = ManaProduction::Fixed([0, 0, 0, 0, 1, 0]);
const MOUNT_OR_VEHICLE: ManaProduction = ManaProduction::Tagged {
    production: &ManaProduction::any_color(2),
    quality: &ManaQuality {
        restriction: ManaRestriction::SpellSubtypeAny(&["Mount", "Vehicle"]),
        spell_bonus: ManaSpellBonus::None,
        retention: ManaRetention::Step,
    },
};

const fn tap_for_mana(program: &'static [Instruction]) -> ActivatedAbility {
    ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        guard: ActivationGuard::Always,
        cycling: false,
        exhaust: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program,
        sorcery: false,
        once_per_turn: false,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::REACH,
    activated: &[
        tap_for_mana(&[Instruction::AddMana(GREEN)]),
        tap_for_mana(&[Instruction::AddMana(MOUNT_OR_VEHICLE)]),
    ],
    ..CardRules::EMPTY
};
