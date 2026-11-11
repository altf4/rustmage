const TAP_MANA: ActivatedAbility = ActivatedAbility {
    exhaust: false,
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    cycling: false,
    cost: ActivationCost {
        tap: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::AddMana(ManaProduction::Tagged {
        production: &ManaProduction::Fixed([1, 0, 0, 0, 0, 0]),
        quality: &ManaQuality {
            restriction: ManaRestriction::ArtifactSpellOrArtifactActivation,
            spell_bonus: ManaSpellBonus::None,
            retention: ManaRetention::Step,
        },
    })],
    sorcery: false,
    once_per_turn: false,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    activated: &[
        TAP_MANA,
        ActivatedAbility {
            cost: ActivationCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 2,
                mana_component: true,
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::ExilePermanentsForTargetManaValue { minimum: None },
                    filter: SacrificeFilter {
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 16,
                        nonland: false,
                        other: true,
                        tokens: None,
                        artifact_enchantment_or_token: false,
                        subtype: None,
                    },
                }),
                ..TAP_MANA.cost
            },
            targets: one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 16,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
            program: &[Instruction::ReturnToBattlefield],
            sorcery: true,
            ..TAP_MANA
        },
    ],
    ..CardRules::EMPTY
};
