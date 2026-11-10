// Direct typed rules from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileEventGraveyardCard],
    }],
    extras: &CardExtras {
        entry_copy: Some(EntryCopySpec {
            filters: &[],
            optional: true,
        }),
        entry_copy_options: Some(&EntryCopyOptions {
            source: EntryCopySource::GraveyardCreature { exile_trigger: 0 },
            name: Some("Superior Spider-Man"),
            exceptions: CopyExceptions {
                base: Some((4, 4)),
                spider_human_hero: true,
                ..CopyExceptions::NONE
            },
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
