// Direct typed rules from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 4),
    keywords: Keywords::MENACE,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Hand,
        cost: ActivationCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 1,
            mana_component: true,
            discard: true,
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::creature_you_control()),
        program: &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 3,
                toughness: 1,
                keywords: Keywords::NONE,
            },
            Instruction::ChangeTargetCreatureForm(AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Additional(&["Dinosaur"]),
                base: None,
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            }),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
