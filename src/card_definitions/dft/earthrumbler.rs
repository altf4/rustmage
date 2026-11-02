// Graveyard exile is a paid nontarget cost; both animation routes expire at cleanup.
const FORM: AnimationSpec = AnimationSpec {
    artifact: true,
    creature_types: AnimationCreatureTypes::Unchanged,
    base: None,
    keywords: Keywords::NONE,
    duration: AnimationDuration::EndOfTurn,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::VIGILANCE.union(Keywords::TRAMPLE),
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost {
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::ExileGraveyard,
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 17,
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
            program: &[Instruction::AnimateSource(FORM)],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            cycling: false,
            origin: ActivationOrigin::Battlefield,
            cost: ActivationCost::ZERO,
            targets: TargetPlan::NONE,
            program: &[Instruction::Crew { minimum: 3 }],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    extras: &CardExtras {
        vehicle_stats: Some(Stats {
            power: 7,
            toughness: 6,
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
