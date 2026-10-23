// Direct typed behavior from full Oracle and complete pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 0, 0],
            generic: 1,
            alternatives: &[],
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::Discard,
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
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::ChooseColor,
            Instruction::SetColors {
                subject: Subject::Source,
                colors: ColorValue::Chosen,
                duration: AnimationDuration::EndOfTurn,
            },
            Instruction::GrantChosenColorHexproof,
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::Source,
            combat_only: true,
            opponent_only: false,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::DrawScaled(
            PlayerSet::Controller,
            AmountValue::Stat(StatValue::Colors(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: false,
            })),
        )],
    }],
    ..CardRules::EMPTY
};
