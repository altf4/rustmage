//! Exact card receipts across combat damage, Craft, linked exile and copying.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn lci(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn mana(card: CardId, ability: u16, color: usize) -> Action {
    let mut production = [0; 6];
    production[color] = 1;
    Action::ActivateMana {
        card,
        ability,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

fn count_tokens(game: &Game, player: Player, card: Card) -> usize {
    game.observe(player)
        .objects
        .iter()
        .filter(|view| {
            view.zone == Zone::Battlefield && view.controller == player && view.card == card
        })
        .count()
}

fn reach(game: &mut Game, predicate: impl Fn(&Game) -> bool) {
    for _ in 0..26 {
        if predicate(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::FinishBlockers | Action::PutTrigger { .. }
            )
        });
    }
    panic!("target decision not reached");
}

fn start_attacks(game: &mut Game, attackers: &[CardId]) {
    reach(game, |game| game.state.pending == Pending::Attackers);
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

#[test]
fn malcolm_hits_a_player_to_gain_chorus_draw_then_cast_the_exact_discard() {
    for player in [Player::P0, Player::P1] {
        let mut root = scenario(player);
        let malcolm = lci(
            &mut root,
            player,
            "Malcolm, Alluring Scoundrel",
            Zone::Battlefield,
        );
        let drawn = put(&mut root, player, "Shock", Zone::Library);
        let unrelated = put(&mut root, player, "Shock", Zone::Graveyard);
        assert!(
            root.keywords(malcolm)
                .contains(Keywords::FLYING.union(Keywords::FLASH))
        );
        for starting in [2, 3] {
            let mut game = root.clone();
            game.add_counter(malcolm, CounterKind::Chorus, starting);
            start_attacks(&mut game, &[malcolm]);
            let discard = Action::Discard { card: drawn };
            reach(&mut game, |game| game.legal_actions().contains(&discard));
            assert_eq!(game.object_view(drawn).zone, Zone::Hand);
            assert_eq!(
                game.counter_count(malcolm, CounterKind::Chorus),
                starting + 1
            );
            act(&mut game, discard);
            if starting == 2 {
                assert!(!matches!(
                    game.state.pending,
                    Pending::ResolutionCasts { .. }
                ));
                settle(&mut game);
                continue;
            }

            let free = game.legal_actions().into_iter().find(|action| {
                matches!(action, Action::CastExiledDuringResolution { object, .. } if object.card == drawn)
            }).unwrap();
            let unrelated = game.object_ref(unrelated);
            reject(
                &mut game,
                Action::CastExiledDuringResolution {
                    object: unrelated,
                    face: CastFace::Normal,
                },
            );
            let mut decline = game.clone();
            act(&mut decline, Action::FinishResolutionCasts);
            settle(&mut decline);
            assert_eq!(
                decline.state.players[player.other().index()].life,
                STARTING_LIFE - 2
            );

            let before_mana = game.state.players[player.index()].mana;
            act(&mut game, free);
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Player(player.other()),
                },
            );
            if matches!(game.state.pending, Pending::PayCosts { .. }) {
                pay(&mut game);
            }
            if matches!(game.state.pending, Pending::ResolutionCasts { .. }) {
                act(&mut game, Action::FinishResolutionCasts);
            }
            settle(&mut game);
            assert_eq!(game.state.players[player.index()].mana, before_mana);
            assert_eq!(
                game.state.players[player.other().index()].life,
                STARTING_LIFE - 4
            );
        }
    }
}

fn pay_artifact_craft(game: &mut Game, source: CardId, material: CardId) {
    act(game, activate(source, 0));
    act(game, Action::BeginOrderedCosts);
    let clause = game
        .observe(game.actor().unwrap())
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::Additional)
        .unwrap() as u16;
    act(game, Action::ChooseCastCost { clause });
    act(
        game,
        Action::ChooseAdditionalCost {
            object: game.object_ref(material),
        },
    );
    for _ in 0..8 {
        match game.state.pending {
            Pending::CastCosts { .. } => choose(game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            }),
            Pending::PayCosts { .. } => pay(game),
            Pending::Priority => return,
            _ => panic!("unexpected Craft cost {:?}", game.state.pending),
        }
    }
    panic!("Craft costs did not finish");
}

#[test]
fn guide_mural_crafts_and_manufactory_checks_both_entry_paths() {
    let golem = Card::from_catalogue_id(5574);
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let source = lci(&mut game, player, "Master's Guide-Mural", Zone::Hand);
        let material = put(&mut game, player, "Swiftfoot Boots", Zone::Graveyard);
        cast(&mut game, source, None);
        settle(&mut game);
        assert_eq!(count_tokens(&game, player, golem), 1);
        let old = game.object_ref(source);
        pay_artifact_craft(&mut game, source, material);
        assert_eq!(game.object_view(source).zone, Zone::Exile);
        assert_eq!(game.object_view(material).zone, Zone::Exile);
        settle(&mut game);
        assert_eq!(game.object_view(source).name, "Master's Manufactory");
        assert_ne!(game.object_ref(source), old);
        act(&mut game, activate(source, 0));
        pay(&mut game);
        settle(&mut game);
        assert!(game.object_view(source).tapped);
        assert_eq!(count_tokens(&game, player, golem), 2);

        let mut exact = scenario(player);
        let entrant = put(&mut exact, player, "Bear Cub", Zone::Hand);
        cast(&mut exact, entrant, None);
        settle(&mut exact);
        let reference = exact.object_ref(entrant);
        let back = Card::from_name("Master's Manufactory").unwrap();
        exact.apply_temporary_copy(reference, CopyRecipe::new(back), CopyDeadline::EndOfTurn);
        assert_eq!(
            exact.object_view(entrant).entry_controller_this_turn,
            Some(player)
        );
        let mut stolen = exact.clone();
        stolen.change_controller(reference, player.other());
        act(&mut stolen, Action::Pass);
        reject(&mut stolen, activate(entrant, 0));
        act(&mut exact, activate(entrant, 0));
        pay(&mut exact);
        settle(&mut exact);
        assert_eq!(count_tokens(&exact, player, golem), 1);

        let mut previous = scenario(player);
        let back = spawn(&mut previous, player, back, Zone::Battlefield);
        previous.state.entered_this_turn = None;
        reject(&mut previous, activate(back, 0));
        let prior = put(&mut previous, player, "Basilisk Collar", Zone::Hand);
        cast(&mut previous, prior, None);
        settle(&mut previous);
        previous.move_object(prior, Zone::Graveyard);
        act(&mut previous, activate(back, 0));
        pay(&mut previous);
        settle(&mut previous);
        assert_eq!(count_tokens(&previous, player, golem), 1);
    }
}

#[test]
fn taq_triples_creatures_and_temple_needs_three_declared_attackers_in_a_main_phase() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let taq = lci(
            &mut game,
            player,
            "Ojer Taq, Deepest Foundation",
            Zone::Battlefield,
        );
        assert!(game.keywords(taq).contains(Keywords::VIGILANCE));
        let evangelist = lci(&mut game, player, "Sanguine Evangelist", Zone::Hand);
        cast(&mut game, evangelist, None);
        settle(&mut game);
        assert_eq!(
            count_tokens(&game, player, Card::from_catalogue_id(5276)),
            3
        );
        let pirate = put(&mut game, player, "Plundering Pirate", Zone::Hand);
        cast(&mut game, pirate, None);
        settle(&mut game);
        assert_eq!(
            count_tokens(&game, player, Card::from_catalogue_id(5302)),
            1
        );
        let removal = put(&mut game, player, "Murder", Zone::Hand);
        cast(&mut game, removal, Some(Target::Card(taq)));
        settle(&mut game);
        assert_eq!(game.object_view(taq).name, "Temple of Civilization");
        assert!(game.object_view(taq).tapped);
        assert_eq!(game.object_view(taq).controller, player);
        game.untap_objects(&[taq], untap_rules::UntapCause::Effect);
        let mut white = game.clone();
        act(&mut white, mana(taq, 0, 0));
        assert!(white.object_view(taq).tapped);
        reject(&mut game, activate(taq, 1));
        let attackers = [0; 3].map(|_| put(&mut game, player, "Bear Cub", Zone::Battlefield));
        for count in [2, 3] {
            let mut branch = game.clone();
            start_attacks(&mut branch, &attackers[..count]);
            branch.state.players[player.index()].mana = [8; 6];
            reject(&mut branch, activate(taq, 1));
            reach(&mut branch, |game| game.state.step == Step::Main2);
            branch.state.players[player.index()].mana = [8; 6];
            if count == 2 {
                reject(&mut branch, activate(taq, 1));
                continue;
            }
            let reference = branch.object_ref(taq);
            act(&mut branch, activate(taq, 1));
            pay(&mut branch);
            settle(&mut branch);
            assert_eq!(branch.object_ref(taq), reference);
            assert_eq!(branch.object_view(taq).name, "Ojer Taq, Deepest Foundation");
            assert!(branch.object_view(taq).tapped);
        }
    }
}

#[test]
fn pit_links_only_its_own_live_exile_targets_and_keeps_a_colorless_ability() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let pit = lci(&mut game, player, "Pit of Offerings", Zone::Hand);
        let blue = put(&mut game, player, "Opt", Zone::Graveyard);
        let red = put(&mut game, player.other(), "Shock", Zone::Graveyard);
        let uncolored = put(&mut game, player.other(), "Forest", Zone::Graveyard);
        let untouched = put(&mut game, player, "Vampire Nighthawk", Zone::Graveyard);
        act(&mut game, Action::PlayLand { card: pit });
        assert!(game.object_view(pit).tapped);
        assert!(game.has_subtype(pit, "Cave"));
        for card in [blue, red, uncolored] {
            act(&mut game, target(card));
        }
        settle(&mut game);
        for card in [blue, red, uncolored] {
            assert_eq!(game.object_view(card).zone, Zone::Exile);
        }
        assert_eq!(game.object_view(untouched).zone, Zone::Graveyard);
        game.untap_objects(&[pit], untap_rules::UntapCause::Effect);
        for (ability, color) in [(0, 5), (1, 1), (1, 3)] {
            let mut branch = game.clone();
            act(&mut branch, mana(pit, ability, color));
            assert!(branch.object_view(pit).tapped);
        }
        reject(&mut game, mana(pit, 1, 2));
        game.move_object(red, Zone::Hand);
        reject(&mut game, mana(pit, 1, 3));
        assert!(game.legal_actions().contains(&mana(pit, 1, 1)));
    }
}

#[test]
fn saheeli_copies_only_copiable_values_and_sacrifices_the_exact_token() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let saheeli = lci(
            &mut game,
            player,
            "Saheeli, the Sun's Brilliance",
            Zone::Battlefield,
        );
        let creature = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let enemy = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        game.add_counter(creature, CounterKind::PlusOne, 1);
        act(&mut game, activate(saheeli, 0));
        reject(&mut game, target(saheeli));
        reject(&mut game, target(enemy));
        act(&mut game, target(creature));
        pay(&mut game);
        assert!(game.object_view(saheeli).tapped);
        settle(&mut game);
        let copy = game
            .observe(player)
            .objects
            .into_iter()
            .find(|view| {
                view.zone == Zone::Battlefield && view.id != creature && view.name == "Bear Cub"
            })
            .unwrap();
        assert_eq!(copy.kinds & 17, 17);
        assert_eq!((copy.power, copy.toughness), (2, 2));
        assert!(copy.keywords.contains(Keywords::HASTE));
        assert_eq!(game.counter_count(copy.id, CounterKind::PlusOne), 0);

        let recipe = game.copiable_recipe(copy.reference);
        let source = game.object_ref(creature);
        game.apply_temporary_copy(source, recipe, CopyDeadline::EndOfTurn);
        assert_eq!(game.object_view(creature).kinds & 17, 17);
        assert!(!game.keywords(creature).contains(Keywords::HASTE));
        game.begin_step(Step::End);
        game.prepare_triggers();
        settle(&mut game);
        assert!(!game.on_battlefield(copy.id));
        assert_eq!(game.object_ref(creature), source);
    }
}

#[test]
fn molten_collapse_only_unlocks_both_after_descending_and_resolves_a_remaining_group() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let spell = lci(&mut game, player, "Molten Collapse", Zone::Hand);
        let sacrifice = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let enemy = put(
            &mut game,
            player.other(),
            "Grizzly Bears",
            Zone::Battlefield,
        );
        let cheap = put(
            &mut game,
            player.other(),
            "Basilisk Collar",
            Zone::Battlefield,
        );
        let expensive = put(
            &mut game,
            player.other(),
            "Swiftfoot Boots",
            Zone::Battlefield,
        );
        let mut early = game.clone();
        act(&mut early, Action::AnnounceCast { card: spell });
        reject(&mut early, Action::ChooseMode { mode: 2 });
        for mode in [0, 1] {
            assert!(early.legal_actions().contains(&Action::ChooseMode { mode }));
        }

        let removal = put(&mut game, player, "Murder", Zone::Hand);
        cast(&mut game, removal, Some(Target::Card(sacrifice)));
        settle(&mut game);
        game.move_object(sacrifice, Zone::Exile);
        act(&mut game, Action::AnnounceCast { card: spell });
        for mode in [0, 1, 2] {
            assert!(game.legal_actions().contains(&Action::ChooseMode { mode }));
        }
        act(&mut game, Action::ChooseMode { mode: 2 });
        reject(&mut game, target(cheap));
        act(&mut game, target(enemy));
        reject(&mut game, target(expensive));
        reject(&mut game, target(enemy));
        act(&mut game, target(cheap));
        pay(&mut game);
        let mut lost_first = game.clone();
        let bounce = put(&mut lost_first, player, "Unsummon", Zone::Hand);
        cast(&mut lost_first, bounce, Some(Target::Card(enemy)));
        settle(&mut lost_first);
        assert_eq!(lost_first.object_view(enemy).zone, Zone::Hand);
        assert_eq!(lost_first.object_view(cheap).zone, Zone::Graveyard);
        settle(&mut game);
        assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
        assert_eq!(game.object_view(cheap).zone, Zone::Graveyard);
    }
}
