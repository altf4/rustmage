// Direct typed behavior from the pinned Oracle and XMage sources.
const fn form(generic: u32, evidence: u64, program: &'static [Instruction]) -> ActivatedAbility {
    ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::CollectEvidence(evidence),
                filter: SacrificeFilter {
                    artifact_enchantment_or_token: false,
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    subtype: None,
                },
            }),
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program,
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    activated: &[
        form(
            1,
            2,
            &[Instruction::ChangeSourceCreatureForm(AnimationSpec {
                artifact: false,
                creature_types: AnimationCreatureTypes::Named(&["Human", "Detective"]),
                base: Some((4, 4)),
                keywords: Keywords::VIGILANCE,
                duration: AnimationDuration::Permanent,
            })],
        ),
        form(
            2,
            4,
            &[
                Instruction::IfSubjectSubtype {
                    subject: Subject::Source,
                    subtype: "Detective",
                    skip: 1,
                },
                Instruction::BecomeNamedLegendaryCreature {
                    name: "Mileva, the Stalwart",
                    base: (5, 5),
                    ability: crate::engine::SelfGrantedAbilityKind::OtherControlledCreatureKeywords(
                        Keywords::INDESTRUCTIBLE,
                    ),
                },
            ],
        ),
    ],
    ..CardRules::EMPTY
};
