// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Artifact,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card::from_catalogue_id(5296),
            amount: 1,
            tapped: false,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ControlledActivationReduction {
            kinds: 16,
            amount: ActivationReductionAmount::Fixed(1),
            leave_one_mana: true,
        }],
    }],
    ..CardRules::EMPTY
};
