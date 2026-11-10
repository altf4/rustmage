//! Paid scenarios for authored Wilds of Eldraine cards, E through L.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const FOOD: Card = Card::from_catalogue_id(5246);
const TREASURE: Card = Card::from_catalogue_id(5302);
const FAERIE: Card = Card::from_catalogue_id(5546);

fn candidate(game: &mut Game, player: Player, name: &str) -> CardId {
    spawn(game, player, Card::from_name(name).unwrap(), Zone::Hand)
}

fn tokens(game: &Game, player: Player, token: Card) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, object)| {
            object.zone == Zone::Battlefield && object.controller == player && object.card == token
        })
        .map(|(id, _)| id as CardId)
        .collect()
}

fn pass_twice(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

#[test]
fn eerie_interference_protects_its_caster_and_later_creatures_from_creatures_this_turn() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(opponent);
        let attacker = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        let shield = candidate(&mut game, player, "Eerie Interference");
        let sailor = put(&mut game, player, "Spectral Sailor", Zone::Hand);
        let bite = put(&mut game, opponent, "Bite Down", Zone::Hand);
        let shock = put(&mut game, opponent, "Shock", Zone::Hand);
        act(&mut game, Action::Pass);
        announce(&mut game, shield, &[]);
        settle(&mut game);
        assert_eq!(game.object_view(shield).zone, Zone::Graveyard);
        assert_eq!(
            game.observe(player).creature_damage_shields[0].scope,
            CreatureDamageShieldScope::PlayerAndCreaturesFromCreatures
        );

        act(&mut game, Action::Pass);
        announce(&mut game, sailor, &[]);
        settle(&mut game);
        announce(
            &mut game,
            bite,
            &[Target::Card(attacker), Target::Card(sailor)],
        );
        settle(&mut game);
        assert_eq!(game.object_view(sailor).zone, Zone::Battlefield);
        assert_eq!(game.object_view(sailor).damage, 0);

        announce(&mut game, shock, &[Target::Player(player)]);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 18);
        pass_twice(&mut game);
        pass_twice(&mut game);
        act(&mut game, Action::Attack { card: attacker });
        act(&mut game, Action::FinishAttackers);
        pass_twice(&mut game);
        act(&mut game, Action::FinishBlockers);
        pass_twice(&mut game);
        assert_eq!(game.state.players[player.index()].life, 18);

        game.state.step = Step::End;
        pass_twice(&mut game);
        assert!(game.observe(player).creature_damage_shields.is_empty());
    }
}

#[test]
fn feed_the_cauldron_checks_mana_value_and_own_turn_independently_of_destruction() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let spell = candidate(&mut game, player, "Feed the Cauldron");
        let target = put(&mut game, opponent, "Redcap Thief", Zone::Battlefield);
        let too_large = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        let protection = put(&mut game, opponent, "Take Up the Shield", Zone::Hand);
        let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);

        act(&mut game, Action::AnnounceCast { card: spell });
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(too_large),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut game);

        let mut indestructible = game.clone();
        act(&mut indestructible, Action::Pass);
        cast(&mut indestructible, protection, Some(Target::Card(target)));
        settle(&mut indestructible);
        assert_eq!(indestructible.object_view(target).zone, Zone::Battlefield);
        assert_eq!(tokens(&indestructible, player, FOOD).len(), 1);

        let mut illegal = game.clone();
        act(&mut illegal, Action::Pass);
        cast(&mut illegal, bounce, Some(Target::Card(target)));
        settle(&mut illegal);
        assert_eq!(illegal.object_view(target).zone, Zone::Hand);
        assert!(tokens(&illegal, player, FOOD).is_empty());

        settle(&mut game);
        assert_eq!(game.object_view(target).zone, Zone::Graveyard);
        let food = tokens(&game, player, FOOD);
        assert_eq!(food.len(), 1);
        act(
            &mut game,
            Action::AnnounceActivate {
                card: food[0],
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut game);
        settle(&mut game);
        assert!(tokens(&game, player, FOOD).is_empty());
        assert_eq!(game.state.players[player.index()].life, 23);

        let mut off_turn = scenario(opponent);
        let spell = candidate(&mut off_turn, player, "Feed the Cauldron");
        let target = put(&mut off_turn, opponent, "Redcap Thief", Zone::Battlefield);
        act(&mut off_turn, Action::Pass);
        announce(&mut off_turn, spell, &[Target::Card(target)]);
        settle(&mut off_turn);
        assert_eq!(off_turn.object_view(target).zone, Zone::Graveyard);
        assert!(tokens(&off_turn, player, FOOD).is_empty());
    }
}

#[test]
fn flick_a_coin_creates_treasure_and_draws_after_prevention_but_needs_a_legal_target() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let spell = candidate(&mut game, player, "Flick a Coin");
        let target = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        let artifact = put(&mut game, opponent, "Swiftfoot Boots", Zone::Battlefield);
        let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        assert!(game.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Player(opponent),
        }));
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(artifact),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut game);

        let mut illegal = game.clone();
        act(&mut illegal, Action::Pass);
        cast(&mut illegal, bounce, Some(Target::Card(target)));
        settle(&mut illegal);
        assert_eq!(illegal.object_view(target).zone, Zone::Hand);
        assert!(tokens(&illegal, player, TREASURE).is_empty());
        assert_eq!(
            illegal.state.players[player.index()].cards_drawn_this_turn,
            0
        );

        let mut prevented = game.clone();
        prevented.add_counter(target, CounterKind::Shield, 1);
        settle(&mut prevented);
        assert_eq!(prevented.object_view(target).damage, 0);
        assert_eq!(prevented.counter_count(target, CounterKind::Shield), 0);
        assert_eq!(tokens(&prevented, player, TREASURE).len(), 1);
        assert_eq!(
            prevented.state.players[player.index()].cards_drawn_this_turn,
            1
        );

        settle(&mut game);
        assert_eq!(game.object_view(target).damage, 1);
        assert_eq!(tokens(&game, player, TREASURE).len(), 1);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    }
}

#[test]
fn food_fight_grants_distinct_paid_artifact_abilities_with_a_live_named_count() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let first = candidate(&mut game, player, "Food Fight");
        let second = candidate(&mut game, player, "Food Fight");
        let opposing = Card::from_name("Food Fight").unwrap();
        let opposing = spawn(&mut game, opponent, opposing, Zone::Battlefield);
        let boots = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
        let opposing_boots = put(&mut game, opponent, "Swiftfoot Boots", Zone::Battlefield);
        let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let removal = put(&mut game, opponent, "Disenchant", Zone::Hand);

        for provider in [first, second] {
            announce(&mut game, provider, &[]);
            settle(&mut game);
        }
        assert!(game.has_subtype(boots, "Equipment"));
        assert!(!game.has_subtype(boots, "Food"));
        let grant = |game: &Game, artifact, provider| Action::AnnounceArtifactFood {
            source: game.object_ref(artifact),
            grantor: game.object_ref(provider),
            ability: 0,
        };
        assert!(game.legal_actions().contains(&grant(&game, boots, second)));
        for (artifact, provider) in [(boots, opposing), (opposing_boots, first), (bear, first)] {
            let action = grant(&game, artifact, provider);
            reject(&mut game, action);
        }
        let sacrificed = game.object_ref(boots);
        let activation = grant(&game, boots, first);
        act(&mut game, activation);
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(opponent),
            },
        );
        pay(&mut game);
        assert_eq!(game.object_view(boots).zone, Zone::Graveyard);
        let source = game.state.stack.last().unwrap().source.unwrap();
        assert_eq!(source.object, sacrificed);
        assert_eq!(source.facts.card.name(), "Swiftfoot Boots");

        let mut answered = game.clone();
        act(&mut answered, Action::Pass);
        cast(&mut answered, removal, Some(Target::Card(first)));
        settle(&mut answered);
        assert_eq!(answered.object_view(first).zone, Zone::Graveyard);
        assert_eq!(answered.state.players[opponent.index()].life, 18);
        settle(&mut game);
        assert_eq!(game.state.players[opponent.index()].life, 17);
    }
}

#[test]
fn faerie_slumber_party_counts_opposing_controllers_and_makes_restricted_fliers() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let spell = candidate(&mut game, player, "Faerie Slumber Party");
        let friendly = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let mut friendly_only = game.clone();
        announce(&mut friendly_only, spell, &[]);
        settle(&mut friendly_only);
        assert_eq!(friendly_only.object_view(friendly).zone, Zone::Hand);
        assert!(tokens(&friendly_only, player, FAERIE).is_empty());

        let stolen = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let exact_stolen = game.object_ref(stolen);
        game.change_controller(exact_stolen, opponent);
        let opposing = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        let equipment = put(&mut game, opponent, "Swiftfoot Boots", Zone::Battlefield);
        announce(&mut game, spell, &[]);
        settle(&mut game);
        for (id, owner) in [(friendly, player), (stolen, player), (opposing, opponent)] {
            let returned = game.object_view(id);
            assert_eq!(returned.zone, Zone::Hand);
            assert_eq!(returned.owner, owner);
        }
        assert_eq!(game.object_view(equipment).zone, Zone::Battlefield);
        let faeries = tokens(&game, player, FAERIE);
        assert_eq!(faeries.len(), 2);
        for &id in &faeries {
            let token = game.object_view(id);
            assert!(token.is_token() && token.has_subtype("Faerie"));
            assert!(token.keywords.contains(Keywords::FLYING));
            assert_eq!(token.colors, 2);
            assert_eq!((token.power, token.toughness), (1, 1));
        }

        let flier = put(&mut game, opponent, "Healer's Hawk", Zone::Battlefield);
        let ground = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        game.state.active = opponent;
        game.state.priority = opponent;
        game.begin_step(Step::BeginCombat);
        pass_twice(&mut game);
        for card in [flier, ground] {
            act(&mut game, Action::Attack { card });
        }
        act(&mut game, Action::FinishAttackers);
        pass_twice(&mut game);
        reject(
            &mut game,
            Action::Block {
                blocker: faeries[0],
                attacker: ground,
            },
        );
        act(
            &mut game,
            Action::Block {
                blocker: faeries[0],
                attacker: flier,
            },
        );
        act(&mut game, Action::FinishBlockers);
    }
}
