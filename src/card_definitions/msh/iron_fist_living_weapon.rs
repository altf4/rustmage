// Full frozen Oracle and direct pinned Java; each resolving cast trigger grants an independent activation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Targeted {
                filter: &CastFilter::Any,
                target: CastTarget::Creature(CastRelation::You),
            },
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantSelfAbility(
            crate::engine::SelfGrantedAbilityKind::Activated { ability: 0 },
        )],
    }],
    extras: &CardExtras {
        self_granted_activation_templates: &[0],
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: false,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Union(&[
            TargetRule::Player {
                opponent_only: false,
            },
            TargetRule::Filtered {
                base: &TargetRule::Any,
                predicate: &TargetPredicate::OtherThanSource,
            },
        ]))),
        program: &[Instruction::DamageScaled(AmountValue::SourcePower)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
