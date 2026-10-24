// Directly authored from the complete pinned Oracle and XMage implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::LandMana {
            recipients: LandManaRecipients::YouControl,
            production: ManaProduction::Tagged {
                production: &ManaProduction::Choice {
                    mask: 31,
                    amount: 2,
                },
                quality: &ManaQuality {
                    restriction: ManaRestriction::InstantSorcery,
                    spell_bonus: ManaSpellBonus::None,
                    retention: ManaRetention::Step,
                },
            },
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::HandSizeAtLeast(7),
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
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
