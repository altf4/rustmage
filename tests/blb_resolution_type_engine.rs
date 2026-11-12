//! Each repeated mass destruction chooses independently during resolution.
mod support;
use rustmage as _;
use rustmage_core::PermanentType;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

const TYPES: &[PermanentType] = &[PermanentType::Artifact, PermanentType::Enchantment];
const TWO_DESTRUCTIONS: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::DestroyAllOfChosenPermanentType(TYPES),
            Instruction::DestroyAllOfChosenPermanentType(TYPES),
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    ..CardRules::EMPTY
};

const ARTIFACT: Action = Action::ChoosePermanentType {
    permanent_type: PermanentType::Artifact,
};
const ENCHANTMENT: Action = Action::ChoosePermanentType {
    permanent_type: PermanentType::Enchantment,
};

#[test]
fn repeated_type_choices_freeze_each_mass_destruction_at_resolution() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Overrun").unwrap(), &TWO_DESTRUCTIONS)]);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let artifacts = [
            put(&mut game, player, "Candy Trail", Zone::Battlefield),
            put(&mut game, opponent, "Swiftfoot Boots", Zone::Battlefield),
        ];
        let enchantments = [
            put(&mut game, player, "Doubling Season", Zone::Battlefield),
            put(&mut game, opponent, "Rest in Peace", Zone::Battlefield),
        ];
        let creature = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        game.add_counter(enchantments[0], CounterKind::Hexproof, 1);
        let spell = put(&mut game, player, "Overrun", Zone::Hand);
        cast(&mut game, spell, None);
        assert_eq!(game.state.pending, Pending::Priority);
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);

        assert_eq!(game.actor(), Some(player));
        assert!(game.observe(opponent).actions.is_empty());
        assert_eq!(
            game.legal_actions(),
            [Action::Concede, ARTIFACT, ENCHANTMENT]
        );
        reject(
            &mut game,
            Action::ChoosePermanentType {
                permanent_type: PermanentType::Creature,
            },
        );
        let saved = game.snapshot();
        let mut indexed = game.clone();
        act(&mut game, ARTIFACT);
        let artifact_index = indexed
            .legal_actions()
            .iter()
            .position(|action| *action == ARTIFACT)
            .unwrap();
        indexed
            .apply_index(player, indexed.revision(), artifact_index)
            .unwrap();
        assert_eq!(game.snapshot(), indexed.snapshot());

        for id in artifacts {
            assert_eq!(game.object_view(id).zone, Zone::Exile);
        }
        for id in enchantments {
            assert_eq!(game.object_view(id).zone, Zone::Battlefield);
        }
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);

        // Repeating the choice remains legal even after the first batch removes
        // every artifact. The rest of the spell runs only after the second choice.
        assert_eq!(
            game.legal_actions(),
            [Action::Concede, ARTIFACT, ENCHANTMENT]
        );
        act(&mut indexed, ARTIFACT);
        for id in enchantments {
            assert_eq!(indexed.object_view(id).zone, Zone::Battlefield);
        }
        assert_eq!(
            indexed.state.players[player.index()].cards_drawn_this_turn,
            1
        );

        act(&mut game, ENCHANTMENT);
        for id in enchantments {
            assert_eq!(game.object_view(id).zone, Zone::Exile);
        }
        assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);

        // Rest in Peace redirects its own simultaneous batch. A later batch
        // samples the battlefield again after that replacement has left.
        indexed.restore(&saved).unwrap();
        assert_eq!(indexed.snapshot(), saved);
        act(&mut indexed, ENCHANTMENT);
        for id in enchantments {
            assert_eq!(indexed.object_view(id).zone, Zone::Exile);
        }
        act(&mut indexed, ARTIFACT);
        for id in artifacts {
            assert_eq!(indexed.object_view(id).zone, Zone::Graveyard);
        }
        assert_eq!(
            indexed.state.players[player.index()].cards_drawn_this_turn,
            1
        );
    }
}
