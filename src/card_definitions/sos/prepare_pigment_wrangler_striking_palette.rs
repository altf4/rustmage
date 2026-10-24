// Whole-card typed Oracle behavior; complete XMage source pinned beside this file.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CopyNextInstantSorcery { ability: 0 }],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::InstantSorcery,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCastSpell {
            prior_spells: false,
        }],
    }],
    extras: &CardExtras {
        prepared_identity: Some("@prepare/pigment_wrangler_striking_palette"),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
