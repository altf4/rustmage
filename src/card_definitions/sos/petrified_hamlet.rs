// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::ChooseLandCardName],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::LandMana {
            recipients: LandManaRecipients::SourceChosenName,
            production: ManaProduction::Fixed([0, 0, 0, 0, 0, 1]),
        }],
    }],
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
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 0, 1,
        ]))],
        sorcery: false,
        once_per_turn: false,
    }],
    extras: &CardExtras {
        named_activation_lock: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
