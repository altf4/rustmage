// Pinned XMage: sacrifice is paid before a simultaneous counter batch on the
// live controlled Klingon cohort.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 2),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            generic: 1,
            mana_component: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::Sacrifice,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1,
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
        program: &[Instruction::ChangeControlledPermanentGroupCounters {
            filters: &[ControlledPermanentFilter {
                all_kinds: 0,
                legendary: false,
                subtype: Some("Klingon"),
            }],
            effect: PermanentGroupCounters::Add {
                kind: CounterKind::PlusOne,
                amount: 1,
            },
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
