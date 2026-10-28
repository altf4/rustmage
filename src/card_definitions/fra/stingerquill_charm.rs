// Directly authored from pinned Oracle and XMage sources; admission awaits rules tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Stingerquill Charm deals 3 damage to any target.",
            targets: one(TargetRule::Any),
            program: &[Instruction::Damage(3)],
        },
        SpellMode {
            text: "Target creature gains first strike and deathtouch until end of turn.",
            targets: one(TargetRule::Creature),
            program: &[Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::FIRST_STRIKE.union(Keywords::DEATHTOUCH),
            }],
        },
        SpellMode {
            text: "Create a 2/2 colorless Wizard Soldier creature token named Cadet. It gains haste until end of turn.",
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokensWithTemporaryKeywords {
                token: Card(5298),
                amount: AmountValue::Fixed(1),
                keywords: Keywords::HASTE,
            }],
        },
    ]),
    ..CardRules::EMPTY
};
