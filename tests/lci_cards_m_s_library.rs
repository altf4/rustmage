//! Printed LCI library effects, artifact mana, and graveyard history.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn lci(game: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, owner, Card::from_name(name).unwrap(), zone)
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn until_choice(game: &mut Game) {
    for _ in 0..10 {
        if game.state.pending != Pending::Priority {
            return;
        }
        act(game, Action::Pass);
    }
    panic!("expected a pending choice");
}

fn activation(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn blue_mana(card: CardId) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: [0, 1, 0, 0, 0, 0],
    }
}

#[test]
fn oaken_siren_funds_artifact_spells_and_artifact_source_activations() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        game.state.players[player.index()].mana = [0; 6];
        let siren = lci(&mut game, player, "Oaken Siren", Zone::Battlefield);
        let artifact = lci(&mut game, player, "Orazca Puzzle-Door", Zone::Hand);
        let opt = put(&mut game, player, "Opt", Zone::Hand);
        assert_eq!(stats(&game, siren), (1, 2));
        assert!(
            game.keywords(siren)
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );
        reject(&mut game, Action::AnnounceCast { card: opt });
        act(&mut game, Action::AnnounceCast { card: artifact });
        act(&mut game, blue_mana(siren));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(artifact).zone, Zone::Battlefield);
        assert!(game.object_view(siren).tapped);

        game.state.objects[siren as usize].tapped = false;
        let first = put(&mut game, player, "Island", Zone::Library);
        let second = put(&mut game, player, "Shock", Zone::Library);
        act(&mut game, activation(artifact));
        act(&mut game, blue_mana(siren));
        pay(&mut game);
        assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
        until_choice(&mut game);
        let look = game.observe(player).library_selection.unwrap();
        assert_eq!(look.minimum, 1);
        assert_eq!(look.remaining.len(), 2);
        assert!(game.observe(player.other()).library_selection.is_none());
        reject(&mut game, Action::FinishLibrarySelection);
        act(&mut game, Action::SelectLibrary { card: first });
        act(&mut game, Action::FinishLibrarySelection);
        assert_eq!(game.object_view(first).zone, Zone::Hand);
        assert_eq!(game.object_view(second).zone, Zone::Graveyard);
    }
}

#[test]
fn sage_private_top_choice_replays_and_declining_mills_all_three() {
    for player in [Player::P0, Player::P1] {
        let mut root = scenario(player);
        let sage = lci(&mut root, player, "Sage of Days", Zone::Hand);
        let first = put(&mut root, player, "Mountain", Zone::Library);
        let second = put(&mut root, player, "Shock", Zone::Library);
        let third = put(&mut root, player, "Bear Cub", Zone::Library);
        cast(&mut root, sage, None);
        until_choice(&mut root);
        assert_eq!(stats(&root, sage), (3, 2));
        let view = root.observe(player).library_selection.unwrap();
        assert_eq!(
            (view.minimum, view.maximum, view.remaining.len()),
            (0, 1, 3)
        );
        assert!(root.observe(player.other()).library_selection.is_none());

        let selected = Action::SelectLibrary { card: second };
        let index = root
            .legal_actions()
            .iter()
            .position(|a| *a == selected)
            .unwrap();
        let revision = root.revision();
        let saved = root.snapshot();
        let mut raw = root.clone();
        let mut indexed = root.clone();
        act(&mut raw, selected);
        indexed.apply_index(player, revision, index).unwrap();
        for branch in [&mut raw, &mut indexed] {
            assert!(branch.observe(player.other()).library_selection.is_none());
            act(branch, Action::FinishLibrarySelection);
            assert_eq!(
                branch.state.players[player.index()].library.last(),
                Some(&second)
            );
            assert_eq!(branch.object_view(first).zone, Zone::Graveyard);
            assert_eq!(branch.object_view(third).zone, Zone::Graveyard);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        assert_eq!(
            indexed.apply_index(player, revision, index),
            Err(Error::StaleRevision)
        );
        act(&mut root, Action::FinishLibrarySelection);
        for card in [first, second, third] {
            assert_eq!(root.object_view(card).zone, Zone::Graveyard);
        }
    }
}

#[test]
fn gnawer_discovers_three_on_death_and_can_put_the_hit_in_hand() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let gnawer = lci(&mut game, player, "Primordial Gnawer", Zone::Battlefield);
        let hit = put(&mut game, player, "Inspiring Overseer", Zone::Library);
        let expensive = put(&mut game, player, "Hill Giant", Zone::Library);
        let land = put(&mut game, player, "Island", Zone::Library);
        assert_eq!(stats(&game, gnawer), (5, 2));
        let murder = put(&mut game, player, "Murder", Zone::Hand);
        cast(&mut game, murder, Some(Target::Card(gnawer)));
        until_choice(&mut game);
        assert_eq!(game.object_view(gnawer).zone, Zone::Graveyard);
        let view = game.observe(player.other()).cascade.unwrap();
        assert_eq!(view.maximum_mana_value, 4);
        assert_eq!(view.discovered, Some(game.object_ref(hit)));
        assert_eq!(
            view.exiled,
            [
                game.object_ref(land),
                game.object_ref(expensive),
                game.object_ref(hit)
            ]
        );
        assert!(
            game.legal_actions()
                .contains(&Action::CastExiledDuringResolution {
                    object: game.object_ref(hit),
                    face: CastFace::Normal
                })
        );
        act(&mut game, Action::FinishResolutionCasts);
        assert_eq!(game.object_view(hit).zone, Zone::Hand);
        assert_eq!(game.object_view(land).zone, Zone::Library);
        assert_eq!(game.object_view(expensive).zone, Zone::Library);
    }
}

#[test]
fn ruin_lurker_checks_descended_history_only_at_its_controllers_end_step() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let bat = lci(&mut game, player, "Ruin-Lurker Bat", Zone::Battlefield);
        let top = put(&mut game, player, "Shock", Zone::Library);
        let departed = put(&mut game, player, "Bear Cub", Zone::Hand);
        assert_eq!(stats(&game, bat), (1, 1));
        assert!(
            game.keywords(bat)
                .contains(Keywords::FLYING.union(Keywords::LIFELINK))
        );
        game.state.step = Step::Main2;

        let mut empty = game.clone();
        pass_pair(&mut empty);
        assert_eq!(empty.state.step, Step::End);
        assert!(empty.state.stack.is_empty());

        game.move_object(departed, Zone::Graveyard);
        game.move_object(departed, Zone::Exile);
        let mut opponent = game.clone();
        opponent.state.active = player.other();
        opponent.state.priority = player.other();
        pass_pair(&mut opponent);
        assert_eq!(opponent.state.step, Step::End);
        assert!(opponent.state.stack.is_empty());

        pass_pair(&mut game);
        until_choice(&mut game);
        act(
            &mut game,
            Action::ArrangeLibrary {
                card: top,
                destination: LibraryDestination::Bottom,
            },
        );
        assert_eq!(
            game.state.players[player.index()].library.first(),
            Some(&top)
        );
    }
}

#[test]
fn surveyor_finds_a_basic_or_cave_and_puts_it_onto_the_battlefield_tapped() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let surveyor = lci(&mut game, player, "Scampering Surveyor", Zone::Hand);
        let basic = put(&mut game, player, "Island", Zone::Library);
        let cave = put(&mut game, player, "Promising Vein", Zone::Library);
        let ineligible = put(&mut game, player, "Tranquil Cove", Zone::Library);
        cast(&mut game, surveyor, None);
        until_choice(&mut game);
        assert_eq!(stats(&game, surveyor), (3, 2));
        reject(&mut game, Action::ChooseSearch { card: ineligible });
        for card in [basic, cave] {
            let mut branch = game.clone();
            act(&mut branch, Action::ChooseSearch { card });
            act(&mut branch, Action::FinishSearch);
            let entry = branch.object_view(card);
            assert_eq!(entry.zone, Zone::Battlefield);
            assert!(entry.tapped);
            assert_eq!(entry.controller, player);
        }
        act(&mut game, Action::FinishSearch);
        for card in [basic, cave, ineligible] {
            assert_eq!(game.object_view(card).zone, Zone::Library);
        }
    }
}
