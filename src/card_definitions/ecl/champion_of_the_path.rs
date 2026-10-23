// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(7, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::OtherSubtypeEntersYou("Elemental"),
            targets: TargetPlan::NONE,
            program: &[Instruction::EnteringPowerDamagePlayers(PlayerSet::Opponent)],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfLeavesBattlefield,
            targets: TargetPlan::NONE,
            program: &[Instruction::ReturnLinkedExileToHand],
        },
    ],

    extras: &CardExtras {
        additional_cost: Some(AdditionalCost {
            kind: EffectChoiceKind::BeholdAndExile,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 0,
                nonland: false,
                other: true,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: Some("Elemental"),
            },
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
