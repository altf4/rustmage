pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::InstantSorcery,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "That spell gains deathtouch and lifelink.",
                targets: TargetPlan::NONE,
                program: &[Instruction::GrantCastSpellKeywords(
                    Keywords::DEATHTOUCH.union(Keywords::LIFELINK),
                )],
            },
            SpellMode {
                text: "Create a 2/2 red Imp creature token.",
                targets: TargetPlan::NONE,
                program: &[Instruction::CreateTokens {
                    token: Card(5684),
                    amount: 1,
                    tapped: false,
                }],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
