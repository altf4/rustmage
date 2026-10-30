// Direct typed rules from pinned Oracle and XMage; gated pending tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::SourceUnprepared,
            event: &TriggerCondition::AtStep {
                step: TriggerStep::Upkeep,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PrepareSource],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        targets: one(TargetRule::Object {
            zone: TargetZone::Graveyard,
            kinds: 0,
            relation: 0,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 2,
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
        program: &[Instruction::BottomLibrary],
    }],
    extras: &CardExtras {
        enters_prepared: false,
        prepared_spell: Some(Card(5450)),
        cost_alternatives: &[ManaCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 1,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
