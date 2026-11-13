// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 1, 0, 0, 0],
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Surveil(1),
            Instruction::CreateCopyTokenWithExceptions {
                subject: Subject::Source,
                amount: 1,
                exceptions: CopyExceptions::NONE,
            },
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::Sacrifice,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    artifact_enchantment_or_token: false,
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1 | 16,
                    nonland: false,
                    other: false,
                    tokens: None,
                    subtype: None,
                },
                amount: 1,
                optional_skip: None,
            },
        ],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
