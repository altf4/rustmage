// Direct typed behavior from complete pinned Oracle and XMage implementations.
const fn ability(
    generic: u32,
    once_per_turn: bool,
    program: &'static [Instruction],
) -> ActivatedAbility {
    ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0; 6],
            generic,
            alternatives: &[],
            additional: None,
            mana_component: false,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program,
        sorcery: false,
        once_per_turn,
    }
}
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::FLYING,
    activated: &[
        ability(
            2,
            false,
            &[Instruction::Grow {
                subject: Subject::Source,
                power: 1,
                toughness: 0,
            }],
        ),
        ability(
            0,
            true,
            &[
                Instruction::ActivationX,
                Instruction::DestroyCombatDamagedPlayersPermanents,
            ],
        ),
    ],
    ..CardRules::EMPTY
};
