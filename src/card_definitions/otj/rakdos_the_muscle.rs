// Complete typed behavior from the pinned Oracle and XMage implementation.
const OTHER_CREATURE: SacrificeFilter = SacrificeFilter {
    keywords: Keywords::NONE,
    source_only: false,
    kinds: 1,
    nonland: false,
    other: true,
    tokens: None,
    artifact_enchantment_or_token: false,
    subtype: None,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 5),
    keywords: Keywords::FLYING.union(Keywords::TRAMPLE),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Sacrificed {
            filter: OTHER_CREATURE,
            own_turn: false,
        },
        targets: one(TargetRule::Player {
            opponent_only: false,
        }),
        program: &[
            Instruction::ExileTopLibraryRemember {
                players: PlayerSet::Targeted,
                amount: AmountValue::SacrificeEventManaValue,
            },
            Instruction::GrantExiledResultFlexiblePlayUntilNextEndStep,
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::Sacrifice,
                filter: OTHER_CREATURE,
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::GrantKeywords {
                subject: Subject::Source,
                keywords: Keywords::INDESTRUCTIBLE,
            },
            Instruction::TapSource,
        ],
        sorcery: false,
        once_per_turn: true,
    }],
    ..CardRules::EMPTY
};
