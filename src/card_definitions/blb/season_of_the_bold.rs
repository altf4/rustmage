// The 16 legal multiplicities of the printed pawprint modes.
const TREASURE: Instruction = Instruction::CreateTokens {
    token: Card(5302),
    amount: 1,
    tapped: true,
};
const PLAY: Instruction = Instruction::ExileTopPlay {
    amount: 2,
    next_turn: true,
};
const DAMAGE: Instruction = Instruction::ScheduleCastTriggerUntilEndOfNextTurn { ability: 0 };

const fn mode(text: &'static str, program: &'static [Instruction]) -> SpellMode {
    SpellMode {
        text,
        targets: TargetPlan::NONE,
        program,
    }
}

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        mode("Choose no modes.", &[]),
        mode("Create a tapped Treasure.", &[TREASURE]),
        mode("Create a tapped Treasure twice.", &[TREASURE, TREASURE]),
        mode(
            "Create a tapped Treasure three times.",
            &[TREASURE, TREASURE, TREASURE],
        ),
        mode(
            "Create a tapped Treasure four times.",
            &[TREASURE, TREASURE, TREASURE, TREASURE],
        ),
        mode(
            "Create a tapped Treasure five times.",
            &[TREASURE, TREASURE, TREASURE, TREASURE, TREASURE],
        ),
        mode("Exile and play two cards.", &[PLAY]),
        mode(
            "Create a tapped Treasure, then exile and play two cards.",
            &[TREASURE, PLAY],
        ),
        mode(
            "Create a tapped Treasure twice, then exile and play two cards.",
            &[TREASURE, TREASURE, PLAY],
        ),
        mode(
            "Create a tapped Treasure three times, then exile and play two cards.",
            &[TREASURE, TREASURE, TREASURE, PLAY],
        ),
        mode("Exile and play two cards twice.", &[PLAY, PLAY]),
        mode(
            "Create a tapped Treasure, then exile and play two cards twice.",
            &[TREASURE, PLAY, PLAY],
        ),
        mode("Set up the spell-cast damage trigger.", &[DAMAGE]),
        mode(
            "Create a tapped Treasure, then set up the spell-cast damage trigger.",
            &[TREASURE, DAMAGE],
        ),
        mode(
            "Create a tapped Treasure twice, then set up the spell-cast damage trigger.",
            &[TREASURE, TREASURE, DAMAGE],
        ),
        mode(
            "Exile and play two cards, then set up the spell-cast damage trigger.",
            &[PLAY, DAMAGE],
        ),
    ]),
    mode_counts: Some(&[0, 1, 2, 3, 4, 5, 1, 2, 3, 4, 2, 3, 1, 2, 3, 2]),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Creature,
            min: 0,
            max: 1,
        }),
        program: &[Instruction::Damage(2)],
    }],
    ..CardRules::EMPTY
};
