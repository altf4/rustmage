//! Card scenarios for the LCI creature and attack-trigger family.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn lci(game: &mut Game, player: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), zone)
}

fn tokens(game: &Game, player: Player, recipe: u16) -> Vec<CardId> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|view| {
            view.zone == Zone::Battlefield
                && view.controller == player
                && view.card == Card::from_catalogue_id(recipe)
        })
        .map(|view| view.id)
        .collect()
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn order_triggers(game: &mut Game) {
    for _ in 0..8 {
        if !matches!(game.state.pending, Pending::OrderTriggers { .. }) {
            return;
        }
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
    panic!("trigger ordering did not finish");
}

fn reach_choice(game: &mut Game, action: &Action) {
    for _ in 0..10 {
        if game.legal_actions().contains(action) {
            return;
        }
        order_triggers(game);
        act(game, Action::Pass);
    }
    panic!("missing decision {action:?}");
}

fn enter_combat(game: &mut Game) {
    pass_pair(game);
    pass_pair(game);
    assert_eq!(game.state.pending, Pending::Attackers);
}

fn attack(game: &mut Game, attackers: &[CardId]) {
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

#[test]
fn hatcher_creates_eggs_and_uses_a_mandatory_combat_choice() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let hatcher = lci(&mut game, player, "Palani's Hatcher", Zone::Hand);
        let foe = put(
            &mut game,
            player.other(),
            "Rampaging Ceratops",
            Zone::Battlefield,
        );
        cast(&mut game, hatcher, None);
        settle(&mut game);
        let eggs = tokens(&game, player, 5571);
        assert_eq!(eggs.len(), 2);
        assert!(!game.keywords(hatcher).contains(Keywords::HASTE));
        assert!(!game.keywords(foe).contains(Keywords::HASTE));
        for egg in &eggs {
            assert_eq!(stats(&game, *egg), (0, 1));
            assert!(game.keywords(*egg).contains(Keywords::HASTE));
        }

        pass_pair(&mut game);
        assert_eq!(game.state.step, Step::BeginCombat);
        let mut absent = game.clone();
        for &egg in &eggs {
            absent.move_object(egg, Zone::Graveyard);
        }
        settle(&mut absent);
        assert!(tokens(&absent, player, 5327).is_empty());

        let sacrifice = Action::Sacrifice {
            object: game.object_ref(eggs[1]),
        };
        reach_choice(&mut game, &sacrifice);
        reject(&mut game, Action::DeclineEffectPayment);
        act(&mut game, sacrifice);
        settle(&mut game);
        assert_eq!(tokens(&game, player, 5571), [eggs[0]]);
        let dinosaurs = tokens(&game, player, 5327);
        assert_eq!(dinosaurs.len(), 1);
        assert_eq!(stats(&game, dinosaurs[0]), (3, 3));
        assert!(game.keywords(dinosaurs[0]).contains(Keywords::HASTE));
    }
}

#[test]
fn ingenuity_uses_declaration_count_and_limits_its_artifact_cast_trigger() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        lci(&mut game, player, "Poetic Ingenuity", Zone::Battlefield);
        let dino1 = put(&mut game, player, "Rampaging Ceratops", Zone::Battlefield);
        let dino2 = put(&mut game, player, "Pathfinding Axejaw", Zone::Battlefield);
        let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        for artifact in ["Basilisk Collar", "Swiftfoot Boots"] {
            let card = put(&mut game, player, artifact, Zone::Hand);
            cast(&mut game, card, None);
            settle(&mut game);
        }
        let new = tokens(&game, player, 5570);
        assert_eq!(new.len(), 1);
        assert_eq!(stats(&game, new[0]), (3, 1));

        enter_combat(&mut game);
        reject(&mut game, Action::Attack { card: new[0] });
        attack(&mut game, &[dino1, dino2, bear]);
        game.move_object(dino1, Zone::Graveyard);
        settle(&mut game);
        assert_eq!(tokens(&game, player, 5302).len(), 2);
        assert_eq!(tokens(&game, player, 5570).len(), 1);
    }
}

#[test]
fn preacher_ties_enable_both_triggers_and_attacking_a_walker_enables_only_the_draw() {
    for player in [Player::P0, Player::P1] {
        let mut root = scenario(player);
        let preacher = lci(
            &mut root,
            player,
            "Preacher of the Schism",
            Zone::Battlefield,
        );
        let walker = put(
            &mut root,
            player.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        root.add_counter(walker, CounterKind::Loyalty, 4);
        assert!(root.keywords(preacher).contains(Keywords::DEATHTOUCH));
        enter_combat(&mut root);
        for (defender, expected) in [
            (Action::Attack { card: preacher }, 1),
            (
                Action::AttackPlaneswalker {
                    card: preacher,
                    defender: root.object_ref(walker),
                },
                0,
            ),
        ] {
            let mut game = root.clone();
            act(&mut game, defender);
            act(&mut game, Action::FinishAttackers);
            order_triggers(&mut game);
            game.gain_life(player.other(), 3);
            settle(&mut game);
            let vampires = tokens(&game, player, 5575);
            assert_eq!(vampires.len(), expected);
            if let Some(&token) = vampires.first() {
                assert_eq!(stats(&game, token), (1, 1));
                assert!(game.keywords(token).contains(Keywords::LIFELINK));
            }
            assert_eq!(game.hand_count(player), 1);
            assert_eq!(game.state.players[player.index()].life, STARTING_LIFE - 1);
        }
    }
}

#[test]
fn hammerskull_samples_other_dinosaurs_at_declaration() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let skull = lci(
            &mut game,
            player,
            "Pugnacious Hammerskull",
            Zone::Battlefield,
        );
        let foe = put(
            &mut game,
            player.other(),
            "Rampaging Ceratops",
            Zone::Battlefield,
        );
        assert_eq!(stats(&game, skull), (6, 6));
        assert!(game.has_subtype(foe, "Dinosaur"));
        let mut companion = game.clone();
        put(
            &mut companion,
            player,
            "Rampaging Ceratops",
            Zone::Battlefield,
        );

        enter_combat(&mut game);
        attack(&mut game, &[skull]);
        put(&mut game, player, "Rampaging Ceratops", Zone::Battlefield);
        settle(&mut game);
        assert_eq!(game.counter_count(skull, CounterKind::Stun), 1);

        enter_combat(&mut companion);
        attack(&mut companion, &[skull]);
        settle(&mut companion);
        assert_eq!(companion.counter_count(skull, CounterKind::Stun), 0);
    }
}

#[test]
fn paladin_entry_and_attack_return_only_own_vampires_with_finality_and_life_loss() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let paladin = lci(&mut game, player, "Queen's Bay Paladin", Zone::Hand);
        let vampire = put(&mut game, player, "Vampire Nighthawk", Zone::Graveyard);
        let enemy = put(
            &mut game,
            player.other(),
            "Vampire Nighthawk",
            Zone::Graveyard,
        );
        let nonvampire = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        cast(&mut game, paladin, None);
        pass_pair(&mut game);
        let choice = Action::ChooseTarget {
            target: Target::Card(vampire),
        };
        assert!(game.legal_actions().contains(&choice));
        for invalid in [enemy, nonvampire] {
            reject(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(invalid),
                },
            );
        }
        let mut declined = game.clone();
        act(&mut declined, Action::FinishTargets);
        settle(&mut declined);
        assert_eq!(declined.object_view(vampire).zone, Zone::Graveyard);
        assert_eq!(declined.state.players[player.index()].life, STARTING_LIFE);

        act(&mut game, choice.clone());
        settle(&mut game);
        assert_eq!(game.object_view(vampire).zone, Zone::Battlefield);
        assert_eq!(game.counter_count(vampire, CounterKind::Finality), 1);
        assert_eq!(game.state.players[player.index()].life, STARTING_LIFE - 3);
        let removal = put(&mut game, player, "Murder", Zone::Hand);
        cast(&mut game, removal, Some(Target::Card(vampire)));
        settle(&mut game);
        assert_eq!(game.object_view(vampire).zone, Zone::Exile);

        let mut attack_game = scenario(player);
        let source = lci(
            &mut attack_game,
            player,
            "Queen's Bay Paladin",
            Zone::Battlefield,
        );
        let vampire = put(
            &mut attack_game,
            player,
            "Vampire Nighthawk",
            Zone::Graveyard,
        );
        enter_combat(&mut attack_game);
        attack(&mut attack_game, &[source]);
        act(
            &mut attack_game,
            Action::ChooseTarget {
                target: Target::Card(vampire),
            },
        );
        settle(&mut attack_game);
        assert_eq!(attack_game.counter_count(vampire, CounterKind::Finality), 1);
    }
}

#[test]
fn resplendent_angel_turns_its_lifelink_damage_into_an_end_step_token() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player.other());
        let angel = lci(&mut game, player, "Resplendent Angel", Zone::Battlefield);
        assert_eq!(stats(&game, angel), (3, 3));
        assert!(game.keywords(angel).contains(Keywords::FLYING));

        let mut short = game.clone();
        short.gain_life(player, 4);
        short.begin_step(Step::End);
        short.prepare_triggers();
        settle(&mut short);
        assert!(tokens(&short, player, 5569).is_empty());

        act(&mut game, Action::Pass);
        act(
            &mut game,
            Action::AnnounceActivate {
                card: angel,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut game);
        settle(&mut game);
        assert_eq!(stats(&game, angel), (5, 5));
        assert!(game.keywords(angel).contains(Keywords::LIFELINK));
        game.apply_damage_batch(&[DamageEvent {
            source: game.damage_source(angel),
            target: Target::Player(player.other()),
            amount: 5,
            combat: false,
            preventable: true,
        }]);
        game.lose_life(player, 2);
        game.begin_step(Step::End);
        game.prepare_triggers();
        settle(&mut game);
        let angels = tokens(&game, player, 5569);
        assert_eq!(angels.len(), 1);
        assert_eq!(stats(&game, angels[0]), (4, 4));
        assert!(
            game.keywords(angels[0])
                .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
        );
    }
}

#[test]
fn throne_adds_its_chosen_type_and_doubles_another_creatures_trigger() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let throne = lci(&mut game, player, "Roaming Throne", Zone::Hand);
        cast(&mut game, throne, None);
        pass_pair(&mut game);
        let choice = Action::ChooseEntryValue {
            value: EntryChoiceValue::CreatureType(CreatureType::from_name("Vampire").unwrap()),
        };
        let index = game
            .legal_actions()
            .iter()
            .position(|action| *action == choice)
            .unwrap();
        let checkpoint = game.snapshot();
        let mut indexed = game.clone();
        act(&mut game, choice.clone());
        indexed
            .apply_index(player, indexed.revision(), index)
            .unwrap();
        assert_eq!(indexed.snapshot(), game.snapshot());
        indexed.restore(&checkpoint).unwrap();
        act(&mut indexed, choice);
        assert_eq!(indexed.snapshot(), game.snapshot());
        assert!(game.has_subtype(throne, "Golem"));
        assert!(game.has_subtype(throne, "Vampire"));

        let priest = lci(&mut game, player, "Sanguine Evangelist", Zone::Hand);
        cast(&mut game, priest, None);
        pass_pair(&mut game);
        order_triggers(&mut game);
        settle(&mut game);
        assert_eq!(tokens(&game, player, 5276).len(), 2);

        let murder = put(&mut game, player.other(), "Murder", Zone::Hand);
        act(&mut game, Action::Pass);
        cast(&mut game, murder, Some(Target::Card(throne)));
        reach_choice(&mut game, &Action::DeclineResolutionMana);
        act(&mut game, Action::DeclineResolutionMana);
        settle(&mut game);
        assert_eq!(game.object_view(throne).zone, Zone::Battlefield);
        assert_eq!(game.object_view(murder).zone, Zone::Graveyard);
    }
}

#[test]
fn evangelist_creates_bats_on_entry_and_death_and_grows_only_other_attackers() {
    let player = Player::P0;
    let mut game = scenario(player);
    let priest = lci(&mut game, player, "Sanguine Evangelist", Zone::Hand);
    cast(&mut game, priest, None);
    settle(&mut game);
    let bats = tokens(&game, player, 5276);
    assert_eq!(bats.len(), 1);
    assert_eq!(stats(&game, bats[0]), (1, 1));
    assert!(game.keywords(bats[0]).contains(Keywords::FLYING));

    let veteran = lci(&mut game, player, "Sanguine Evangelist", Zone::Battlefield);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let reserve = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    enter_combat(&mut game);
    attack(&mut game, &[veteran, ally]);
    settle(&mut game);
    assert_eq!(stats(&game, veteran), (2, 1));
    assert_eq!(stats(&game, ally), (3, 2));
    assert_eq!(stats(&game, reserve), (2, 2));

    game.move_object(veteran, Zone::Graveyard);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(tokens(&game, player, 5276).len(), 2);
}
