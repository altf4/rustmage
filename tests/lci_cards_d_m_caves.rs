//! The five hidden Caves share a paid, sorcery-speed discover activation.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

#[test]
fn hidden_caves_enter_tapped_produce_their_colors_and_sacrifice_to_discover_four() {
    for (name, color, player) in [
        ("Hidden Cataract", 1, Player::P0),
        ("Hidden Courtyard", 0, Player::P1),
        ("Hidden Necropolis", 2, Player::P0),
        ("Hidden Nursery", 4, Player::P1),
        ("Hidden Volcano", 3, Player::P0),
    ] {
        let mut game = scenario(player);
        let land = spawn(
            &mut game,
            player,
            Card::from_name(name).unwrap(),
            Zone::Hand,
        );
        let found = put(&mut game, player, "Galvanic Giant", Zone::Library);
        let expensive = put(&mut game, player, "Gigantosaurus", Zone::Library);
        let skipped = put(&mut game, player, "Forest", Zone::Library);
        game.state.players[player.index()].mana = [0; 6];
        let mut color_mana = [0; 6];
        color_mana[color] = 1;
        let mana = Action::ActivateMana {
            card: land,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: color_mana,
        };
        let discover = Action::AnnounceActivate {
            card: land,
            ability: 1,
            counter_payment: Counters::default(),
        };

        act(&mut game, Action::PlayLand { card: land });
        assert!(game.has_subtype(land, "Cave"), "{name}");
        assert!(game.object_view(land).tapped, "{name}");
        reject(&mut game, mana.clone());
        game.untap_objects(&[land], untap_rules::UntapCause::Effect);
        act(&mut game, mana);
        assert_eq!(
            game.state.players[player.index()].mana,
            color_mana,
            "{name}"
        );
        game.state.players[player.index()].mana[5] = 4;
        reject(&mut game, discover.clone());
        game.untap_objects(&[land], untap_rules::UntapCause::Effect);
        let mut wrong_color = game.clone();
        wrong_color.clear_step_mana(false, false);
        wrong_color.state.players[player.index()].mana = [0, 0, 0, 0, 0, 5];
        reject(&mut wrong_color, discover.clone());
        game.state.step = Step::Upkeep;
        reject(&mut game, discover.clone());
        game.state.step = Step::Main1;
        act(&mut game, discover);
        pay(&mut game);
        assert_eq!(game.object_view(land).zone, Zone::Graveyard, "{name}");
        assert_eq!(game.state.players[player.index()].mana, [0; 6], "{name}");
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        let event = game.observe(player.other()).cascade.unwrap();
        assert_eq!(
            event.exiled,
            [skipped, expensive, found].map(|id| game.object_ref(id)),
            "{name}"
        );
        assert_eq!(event.discovered, Some(game.object_ref(found)), "{name}");
        act(&mut game, Action::FinishResolutionCasts);
        assert_eq!(game.object_view(found).zone, Zone::Hand, "{name}");
        let library = &game.state.players[player.index()].library;
        assert!(
            library[..2].contains(&skipped) && library[..2].contains(&expensive),
            "{name}"
        );
    }
}
