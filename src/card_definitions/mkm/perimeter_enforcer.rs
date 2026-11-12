// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    keywords: Keywords::FLYING.union(Keywords::LIFELINK),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FilteredEntersOrTurnsFaceUpYou {
            entry: EntryPredicate::SubtypeAny(&["Detective"]),
            other_on_entry: true,
            face_up_subtype: "Detective",
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Grow {
            subject: Subject::Source,
            power: 1,
            toughness: 1,
        }],
    }],
    ..CardRules::EMPTY
};
