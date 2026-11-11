pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&TriggerCondition::CommittedCrime),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            },
            Instruction::ChooseObjects {
                kind: EffectChoiceKind::ExileCardFromAnyGraveyardRemember,
                players: PlayerSet::Controller,
                filter: SacrificeFilter {
                    kinds: 0,
                    keywords: Keywords::NONE,
                    source_only: false,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
                amount: 1,
                optional_skip: Some(0),
            },
            Instruction::MayIfRememberedExiledCreature { skip: 1 },
            Instruction::CopySourceFromRememberedExiledCreature,
        ],
    }],
    ..CardRules::EMPTY
};
