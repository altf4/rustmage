pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::MoveBattlefield {
                filters: &[BattlefieldFilter {
                    kinds: 16,
                    relation: 1,
                    nonland: false,
                    tapped: None,
                    without: Keywords::NONE,
                    subtype: None,
                    max_mana: u32::MAX,
                }],
                exile: false,
            },
            Instruction::ExileTopLibraryRemember {
                players: PlayerSet::Controller,
                amount: AmountValue::Stat(
                    StatValue::ArtifactsPutIntoGraveyardsFromBattlefieldThisTurn,
                ),
            },
            Instruction::SelectMilled(MilledSelectionSpec {
                filter: MilledFilter::Kinds(1),
                count: MilledCount::Range {
                    minimum: 0,
                    maximum: 1,
                },
                destination: MilledDestination::BattlefieldWithHasteAndReturn { ability: 0 },
            }),
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::ReturnDelayedBattlefieldObjectsToHand],
    }],
    ..CardRules::EMPTY
};
