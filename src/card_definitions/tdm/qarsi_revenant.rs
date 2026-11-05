// Source-audited direct typed definition; see the paired provenance record.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    keywords: Keywords::FLYING
        .union(Keywords::DEATHTOUCH)
        .union(Keywords::LIFELINK),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Graveyard,
        cost: ActivationCost {
            generic: 2,
            colored: [0, 0, 1, 0, 0, 0],
            mana_component: true,
            exile_self: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[Instruction::PutCountersMixed {
            subject: Subject::Targets,
            amounts: &[
                (CounterKind::Flying, 1),
                (CounterKind::Deathtouch, 1),
                (CounterKind::Lifelink, 1),
            ],
        }],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
