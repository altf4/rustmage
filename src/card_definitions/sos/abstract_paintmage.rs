// Direct typed behavior from the complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Main1,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Fixed([0, 1, 0, 1, 0, 0]),
            quality: &ManaQuality {
                restriction: ManaRestriction::InstantSorcery,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Step,
            },
        })],
    }],
    extras: &CardExtras {
        cost_alternatives: &[ManaCost {
            colored: [0, 2, 0, 1, 0, 0],
            generic: 0,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
