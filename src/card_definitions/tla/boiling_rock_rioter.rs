// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::AddCombatRed(1)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::CastOneOwnedLinkedPaid(CastFilter::Subtype(
                "Ally",
            ))],
        },
    ],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::TapPermanents(1),
                filter: SacrificeFilter {
                    artifact_enchantment_or_token: false,
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: false,
                    tokens: None,
                    subtype: Some("Ally"),
                },
            }),
            ..ActivationCost::ZERO
        },
        targets: one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 0,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::ExileGraveTargetsLinked],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
