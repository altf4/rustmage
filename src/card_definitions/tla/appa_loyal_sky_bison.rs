// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Target creature you control gains flying until end of turn.",
                targets: one(TargetRule::creature_you_control()),
                program: &[Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::FLYING,
                }],
            },
            SpellMode {
                text: "Airbend another target nonland permanent you control.",
                targets: one(TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 0,
                    relation: 0,
                    other: true,
                    nonland: true,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                }),
                program: &[Instruction::AirbendTargets],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
