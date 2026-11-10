// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    keywords: Keywords::LIFELINK,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: 1,
        }),
        program: &[Instruction::GrowScaled {
            subject: Subject::Targets,
            amount: AmountValue::LifeGainedThisTurn,
            power: -1,
            toughness: -1,
        }],
    }],
    extras: &CardExtras {
        adventure_spell: Some(Card(5553)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
