// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::MENACE,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::MENACE)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::NonbasicLandsEnterTapped],
        },
        StaticAbilityGroup {
            line: 3,
            first_layer: AbilityLayer::Type,
            fragments: &[StaticAbilityFragment::NonbasicLandTypeWhileCounter {
                kind: BasicLandType::Mountain,
                counter: CounterKind::Conqueror,
            }],
        },
    ],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        exhaust: false,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            generic: 7,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCounters {
            subject: Subject::Source,
            kind: CounterKind::Conqueror,
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
