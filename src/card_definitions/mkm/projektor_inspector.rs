// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersOrTurnsFaceUpYou {
            entry: EntryPredicate::SourceOrSubtypeAny(&["Detective"]),
            other_on_entry: false,
            face_up_subtype: "Detective",
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayDraw { skip: 2 },
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::Discard(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
