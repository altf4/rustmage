pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::creature_you_control(),
            min: 0,
            max: 1,
        }),
        program: &[Instruction::Suspect {
            subject: Subject::Targets,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 0, 1, 1, 0, 0],
            generic: 3,
            mana_component: true,
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::FilteredCost {
                    action: CostAction::Sacrifice,
                    any: &[CostMatch::Suspected],
                },
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
        targets: one(TargetRule::Creature),
        program: &[Instruction::Grow {
            subject: Subject::Targets,
            power: -5,
            toughness: -5,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
