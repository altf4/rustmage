//! Federation counts effective creature types on controlled non-Borg creatures.
use crate::support::*;

use rustmage as _;
use rustmage_core::cards::authored::{CardRules, one};
use rustmage_core::{cards::*, engine::*};

const TYPES: StatValue = StatValue::DistinctControlledCreatureTypesExcept("Borg");
const MEASURE: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::GainLifeScaled(
            PlayerSet::Controller,
            AmountValue::Stat(TYPES),
        )],
    ),
    ..CardRules::EMPTY
};
const BORGIFY: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::AddCreatureTypes {
            subject: Subject::Targets,
            types: &["Borg"],
        }],
    ),
    ..CardRules::EMPTY
};
const REPLACE_TYPES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::SetCreatureTypes {
            subject: Subject::Targets,
            types: &["Gorn"],
        }],
    ),
    ..CardRules::EMPTY
};
const TAKE_CONTROL: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::Creature),
        &[Instruction::GainControl {
            subject: Subject::Targets,
            controller: PlayerSet::Controller,
        }],
    ),
    ..CardRules::EMPTY
};

#[test]
fn federation_unions_live_types_and_excludes_each_borg_creature() {
    let _rules = fixture_rules::activate(&[
        (Card::from_name("Opt").unwrap(), &MEASURE),
        (Card::from_name("Giant Growth").unwrap(), &BORGIFY),
        (Card::from_name("Unsummon").unwrap(), &REPLACE_TYPES),
        (
            Card::from_name("Involuntary Employment").unwrap(),
            &TAKE_CONTROL,
        ),
    ]);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let elf = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
        let changeling = put(&mut game, player, "Three Tree Mascot", Zone::Battlefield);
        let opposing_giant = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
        assert!(game.has_subtype(changeling, "Borg"));

        let first = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, first, None);
        settle(&mut game);
        // Bear, Elf, Druid: duplicate Bears count once; the Changeling is Borg.
        assert_eq!(game.state.players[player.index()].life, 23);
        assert_eq!(game.state.players[player.other().index()].life, 20);

        let conversion = put(&mut game, player, "Giant Growth", Zone::Hand);
        cast(&mut game, conversion, Some(Target::Card(elf)));
        settle(&mut game);
        assert!(game.has_subtype(elf, "Borg"));
        assert!(game.has_subtype(elf, "Elf"));

        let second = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, second, None);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 24);

        let replacement = put(&mut game, player, "Unsummon", Zone::Hand);
        cast(&mut game, replacement, Some(Target::Card(elf)));
        settle(&mut game);
        assert!(game.has_subtype(elf, "Gorn"));
        assert!(!game.has_subtype(elf, "Borg"));
        assert!(!game.has_subtype(elf, "Elf"));
        let third = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, third, None);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 26);

        let control = put(&mut game, player, "Involuntary Employment", Zone::Hand);
        cast(&mut game, control, Some(Target::Card(opposing_giant)));
        settle(&mut game);
        assert_eq!(game.object_view(opposing_giant).controller, player);
        let fourth = put(&mut game, player, "Opt", Zone::Hand);
        cast(&mut game, fourth, None);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 29);
    }
}
