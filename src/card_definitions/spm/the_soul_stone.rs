// Direct typed rules from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::INDESTRUCTIBLE,
    static_groups: &[StaticAbilityGroup {
        line: 3,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::HarnessedTrigger(0)],
    }],
    activated: &[
        ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 1, 0, 0, 0,
            ]))],
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            sorcery: false,
            once_per_turn: false,
            cycling: false,
            exhaust: false,
        },
        ActivatedAbility {
            cost: ActivationCost {
                colored: [0, 0, 1, 0, 0, 0],
                generic: 6,
                mana_component: true,
                tap: true,
                additional: Some(AdditionalCost {
                    kind: EffectChoiceKind::ExileControlled,
                    filter: SacrificeFilter {
                        kinds: 1,
                        keywords: Keywords::NONE,
                        source_only: false,
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
            program: &[Instruction::HarnessSource],
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            sorcery: false,
            once_per_turn: false,
            cycling: false,
            exhaust: false,
        },
    ],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        }),
        targets: one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 1,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::ReturnToBattlefield],
    }],
    ..CardRules::EMPTY
};
