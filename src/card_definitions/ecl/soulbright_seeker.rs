// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: one(TargetRule::creature_you_control()),
        program: &[
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::TRAMPLE,
            },
            Instruction::IfThisAbilityResolutionNumber { number: 3, skip: 1 },
            Instruction::AddMana(ManaProduction::Fixed([0, 0, 0, 4, 0, 0])),
        ],
        sorcery: false,
        once_per_turn: false,
    }],

    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::Behold,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 0,
                nonland: false,
                other: true,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: Some("Elemental"),
            },
        }),
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 0, 1, 0, 0],
            generic: 2,
        }],
        mana_alternative_replaces_additional: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
