// Direct typed rules from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledSubtypeIgnoresLegendRule(
            "Spider",
        )],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayFromOutsideOwnHand(&TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Any,
            ordinal: None,
        }),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayCopyOncePerTurn { skip: 2 },
            Instruction::RecordOncePerTurn,
            Instruction::CopyCapturedSpellModified {
                retarget: true,
                modification: CapturedSpellModification::PermanentHaste,
            },
        ],
    }],
    ..CardRules::EMPTY
};
