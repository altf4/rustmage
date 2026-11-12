// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    activated: &[ActivatedAbility {
        origin: ActivationOrigin::Battlefield,
        guard: ActivationGuard::OncePerIncarnation(&ActivationGuard::Always),
        cost: ActivationCost {
            generic: 3,
            mana_component: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::Discard,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 3,
            },
            Instruction::AddCharacteristics {
                subject: Subject::Source,
                addition: CharacteristicAddition {
                    card_types: 0,
                    creature_types: &["Demon"],
                    colors: 4,
                    duration: AnimationDuration::Permanent,
                },
            },
        ],
        sorcery: false,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    ..CardRules::EMPTY
};
