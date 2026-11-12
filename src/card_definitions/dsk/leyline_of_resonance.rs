pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::OnlyTargeted {
                filter: &CastFilter::InstantSorcery,
                target: CastTarget::Creature(CastRelation::You),
            },
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCastSpell {
            prior_spells: false,
        }],
    }],
    extras: &CardExtras {
        opening_hand_battlefield: true,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
