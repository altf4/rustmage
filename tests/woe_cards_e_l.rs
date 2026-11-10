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

#[test]
fn extraordinary_journey_uses_paid_x_owner_permissions_and_one_exile_entry_draw() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let journey = candidate(&mut game, player, "Extraordinary Journey");
        let first = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let second = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
        let third = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
        let remove = put(&mut game, player, "Disenchant", Zone::Hand);
        let borrowed = game.object_ref(second);
        game.change_controller(borrowed, opponent);

        let mut zero = game.clone();
        act(&mut zero, Action::AnnounceCast { card: journey });
        act(&mut zero, Action::ChooseX { value: 0 });
        act(
            &mut zero,
            Action::PayCosts {
                payment: [0, 2, 0, 0, 0, 0],
            },
        );
        settle(&mut zero);
        assert!(zero.observe(player).exile_permissions.is_empty());
        assert_eq!(zero.object_view(first).zone, Zone::Battlefield);

        act(&mut game, Action::AnnounceCast { card: journey });
        act(&mut game, Action::ChooseX { value: 2 });
        reject(
            &mut game,
            Action::PayCosts {
                payment: [0, 2, 0, 0, 0, 2],
            },
        );
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 2, 0, 0, 0, 4],
            },
        );
        pass_twice(&mut game);
        assert!(game.legal_actions().contains(&Action::FinishTargets));
        for card in [first, second] {
            act(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(card),
                },
            );
        }
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(third),
            },
        );
        if game.legal_actions().contains(&Action::FinishTargets) {
            act(&mut game, Action::FinishTargets);
        }
        settle(&mut game);
        assert_eq!(game.object_view(third).zone, Zone::Battlefield);
        for card in [first, second] {
            assert_eq!(game.object_view(card).zone, Zone::Exile);
            let exact = game.object_ref(card);
            assert!(game.observe(opponent).exile_permissions.iter().any(|row| {
                row.object == exact
                    && row.player == player
                    && row.expiry == ExilePlayExpiry::WhileExiled
            }));
        }

        let mut departed = game.clone();
        cast(&mut departed, remove, Some(Target::Card(journey)));
        settle(&mut departed);
        assert_eq!(departed.object_view(journey).zone, Zone::Graveyard);
        act(&mut departed, Action::AnnounceExile { card: first });
        pay(&mut departed);
        settle(&mut departed);
        assert_eq!(departed.object_view(first).zone, Zone::Battlefield);
        assert_eq!(
            departed.state.players[player.index()].cards_drawn_this_turn,
            0
        );

        for card in [first, second] {
            act(&mut game, Action::AnnounceExile { card });
            pay(&mut game);
            settle(&mut game);
            assert_eq!(game.object_view(card).zone, Zone::Battlefield);
            assert_eq!(game.object_view(card).controller, player);
            assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
        }
    }
}

#[test]
fn faerie_fencing_remembers_its_cast_faerie_and_a_copy_only_uses_x() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let target = put(&mut game, opponent, "Gigantosaurus", Zone::Battlefield);
        let spell = candidate(&mut game, player, "Faerie Fencing");
        let faerie = put(&mut game, player, "Barrow Naughty", Zone::Battlefield);
        let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);
        let teach = put(&mut game, player, "Teach by Example", Zone::Hand);
        cast(&mut game, teach, None);
        settle(&mut game);

        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseX { value: 1 });
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 1, 0, 0, 1],
            },
        );
        act(&mut game, Action::Pass);
        cast(&mut game, bounce, Some(Target::Card(faerie)));

        for _ in 0..32 {
            if game.object_view(target).power != 10 {
                break;
            }
            choose(&mut game, |action| {
                matches!(
                    action,
                    Action::Pass | Action::FinishStackTargets | Action::PutTrigger { .. }
                )
            });
        }
        assert_eq!(game.object_view(faerie).zone, Zone::Hand);
        assert_eq!(game.object_view(target).power, 9, "copy uses only X");
        settle(&mut game);
        let target = game.object_view(target);
        assert_eq!((target.power, target.toughness), (5, 5));
    }
}

#[test]
fn ingenious_prodigy_enters_with_x_and_its_upkeep_checks_a_live_counter() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let prodigy = candidate(&mut game, player, "Ingenious Prodigy");
        let large = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        let small = put(&mut game, opponent, "Healer's Hawk", Zone::Battlefield);
        let mut zero = game.clone();
        for (branch, x) in [(&mut zero, 0), (&mut game, 1)] {
            act(branch, Action::AnnounceCast { card: prodigy });
            act(branch, Action::ChooseX { value: x });
            act(
                branch,
                Action::PayCosts {
                    payment: [0, 1, 0, 0, 0, x],
                },
            );
            settle(branch);
            assert_eq!(branch.object_view(prodigy).zone, Zone::Battlefield);
            assert_eq!(
                branch.counter_count(prodigy, CounterKind::PlusOne),
                u64::from(x)
            );
            for _ in 0..2 {
                branch.state.step = Step::End;
                pass_twice(branch);
            }
            assert_eq!(branch.state.active, player);
            assert_eq!(branch.state.step, Step::Upkeep);
        }
        assert!(zero.state.stack.is_empty());
        assert!(!game.state.stack.is_empty());
        assert!(game.object_view(prodigy).keywords.contains(Keywords::SKULK));
        assert!(game.can_block_pair(small, prodigy));
        assert!(!game.can_block_pair(large, prodigy));

        let mut removed = game.clone();
        removed.pay_counters(prodigy, &Counters::of(CounterKind::PlusOne, 1));
        settle(&mut removed);
        assert_eq!(
            removed.state.players[player.index()].cards_drawn_this_turn,
            0
        );

        pass_twice(&mut game);
        let mut declined = game.clone();
        act(&mut declined, Action::ChooseOptional { accept: false });
        assert_eq!(declined.counter_count(prodigy, CounterKind::PlusOne), 1);
        assert_eq!(
            declined.state.players[player.index()].cards_drawn_this_turn,
            0
        );
        act(&mut game, Action::ChooseOptional { accept: true });
        assert_eq!(game.counter_count(prodigy, CounterKind::PlusOne), 0);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
        assert!(!game.can_block_pair(small, prodigy));
    }
}

#[test]
fn edgewall_inn_enters_tapped_remembers_its_color_and_recovers_an_owned_adventure() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let inn = candidate(&mut game, player, "Edgewall Inn");
        let adventure = put(&mut game, player, "Frolicking Familiar", Zone::Graveyard);
        let foreign = put(&mut game, opponent, "Frolicking Familiar", Zone::Graveyard);
        let ordinary = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        act(&mut game, Action::PlayLand { card: inn });
        let choice = EntryChoiceValue::Color(ChoiceColor::Blue);
        act(&mut game, Action::ChooseEntryValue { value: choice });
        assert_eq!(game.object_view(inn).entry_choice, Some(choice));
        assert!(game.object_view(inn).tapped);
        let blue = Action::ActivateMana {
            card: inn,
            ability: 0,
            counter_payment: Counters::default(),
            payment: [0; 6],
            production: [0, 1, 0, 0, 0, 0],
        };
        reject(&mut game, blue.clone());
        for _ in 0..2 {
            game.state.step = Step::End;
            pass_twice(&mut game);
        }
        assert_eq!(game.actor(), Some(player));
        assert!(!game.object_view(inn).tapped);
        let mut mana_branch = game.clone();
        act(&mut mana_branch, blue);
        assert_eq!(
            mana_branch.state.players[player.index()].mana,
            [0, 1, 0, 0, 0, 0]
        );
        assert!(mana_branch.object_view(inn).tapped);

        game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
        act(
            &mut game,
            Action::AnnounceActivate {
                card: inn,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        for invalid in [foreign, ordinary] {
            reject(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(invalid),
                },
            );
        }
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(adventure),
            },
        );
        pay(&mut game);
        assert_eq!(game.object_view(inn).zone, Zone::Graveyard);
        settle(&mut game);
        assert_eq!(game.object_view(adventure).zone, Zone::Hand);
        assert_eq!(game.object_view(foreign).zone, Zone::Graveyard);
        assert_eq!(game.object_view(ordinary).zone, Zone::Graveyard);
    }
}

#[test]
fn frantic_firebolt_counts_the_live_owned_graveyard_union() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let spell = candidate(&mut game, player, "Frantic Firebolt");
        let target = put(
            &mut game,
            player.other(),
            "Gigantosaurus",
            Zone::Battlefield,
        );
        let instant = put(&mut game, player, "Shock", Zone::Graveyard);
        put(&mut game, player, "Divination", Zone::Graveyard);
        put(&mut game, player, "Frolicking Familiar", Zone::Graveyard);
        put(&mut game, player, "Bear Cub", Zone::Graveyard);
        put(&mut game, player.other(), "Shock", Zone::Graveyard);
        announce(&mut game, spell, &[Target::Card(target)]);
        let mut changed = game.clone();
        changed.move_object(instant, Zone::Exile);
        settle(&mut changed);
        assert_eq!(changed.object_view(target).damage, 4);
        settle(&mut game);
        assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(game.object_view(target).damage, 5);
    }
}

#[test]
fn howling_galefang_gains_haste_from_a_live_owned_exiled_adventure() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let galefang = candidate(&mut game, player, "Howling Galefang");
        let adventure = put(&mut game, player, "Frolicking Familiar", Zone::Hand);
        put(
            &mut game,
            player.other(),
            "Frolicking Familiar",
            Zone::Exile,
        );
        put(&mut game, player, "Forest", Zone::Exile);
        announce(&mut game, galefang, &[]);
        settle(&mut game);
        let view = game.object_view(galefang);
        assert!(view.keywords.contains(Keywords::VIGILANCE));
        assert!(!view.keywords.contains(Keywords::HASTE));

        game.move_object(adventure, Zone::Exile);
        assert!(game
            .object_view(galefang)
            .keywords
            .contains(Keywords::HASTE));
        let mut removed = game.clone();
        removed.move_object(adventure, Zone::Hand);
        for branch in [&mut game, &mut removed] {
            pass_twice(branch);
            pass_twice(branch);
        }
        let attack = Action::Attack { card: galefang };
        reject(&mut removed, attack.clone());
        act(&mut game, attack);
        act(&mut game, Action::FinishAttackers);
        assert!(!game.object_view(galefang).tapped);
    }
}

#[test]
fn gingerbrute_has_paid_food_and_a_temporary_haste_only_blocking_rule() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let gingerbrute = candidate(&mut game, player, "Gingerbrute");
        let slow = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        let fast = put(&mut game, opponent, "Harried Spearguard", Zone::Battlefield);
        announce(&mut game, gingerbrute, &[]);
        settle(&mut game);
        assert!(game.has_subtype(gingerbrute, "Food"));
        assert!(game
            .object_view(gingerbrute)
            .keywords
            .contains(Keywords::HASTE));

        let activate = |ability| Action::AnnounceActivate {
            card: gingerbrute,
            ability,
            counter_payment: Counters::default(),
        };
        let mut eaten = game.clone();
        act(&mut eaten, activate(1));
        act(
            &mut eaten,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 0, 2],
            },
        );
        assert_eq!(eaten.object_view(gingerbrute).zone, Zone::Graveyard);
        settle(&mut eaten);
        assert_eq!(eaten.state.players[player.index()].life, 23);

        act(&mut game, activate(0));
        act(
            &mut game,
            Action::PayCosts {
                payment: [0, 0, 0, 0, 0, 1],
            },
        );
        settle(&mut game);
        let mut expired = game.clone();
        expired.state.step = Step::End;
        pass_twice(&mut expired);
        assert!(!expired
            .object_view(gingerbrute)
            .keywords
            .contains(Keywords::ONLY_HASTE_BLOCKERS));

        pass_twice(&mut game);
        pass_twice(&mut game);
        act(&mut game, Action::Attack { card: gingerbrute });
        act(&mut game, Action::FinishAttackers);
        pass_twice(&mut game);
        reject(
            &mut game,
            Action::Block {
                blocker: slow,
                attacker: gingerbrute,
            },
        );
        act(
            &mut game,
            Action::Block {
                blocker: fast,
                attacker: gingerbrute,
            },
        );
    }
}

#[test]
fn into_the_fae_court_draws_three_and_creates_one_restricted_faerie() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let spell = candidate(&mut game, player, "Into the Fae Court");
        announce(&mut game, spell, &[]);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 3);
        assert_eq!(
            game.state.players[player.other().index()].cards_drawn_this_turn,
            0
        );
        let faeries = tokens(&game, player, FAERIE);
        assert_eq!(faeries.len(), 1);
        assert!(game
            .object_view(faeries[0])
            .keywords
            .contains(Keywords::FLYING));
        assert!(tokens(&game, player.other(), FAERIE).is_empty());
    }
}

fn choose_target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn cast_adventure(game: &mut Game, card: CardId, target: Option<Target>) {
    act(game, Action::AnnounceAdventure { card });
    if let Some(target) = target {
        choose_target(game, target);
    }
    pay(game);
}

fn cast_adventurer(game: &mut Game, card: CardId) {
    act(game, Action::AnnounceExile { card });
    pay(game);
}

#[test]
fn fell_horseman_recovers_an_owned_creature_and_bottoms_only_its_exact_death() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let horseman = candidate(&mut game, player, "Fell Horseman // Deathly Ride");
        let own = put(&mut game, player, "Bear Cub", Zone::Graveyard);
        let other = put(&mut game, player.other(), "Bear Cub", Zone::Graveyard);
        let murder = put(&mut game, player, "Murder", Zone::Hand);
        act(&mut game, Action::AnnounceAdventure { card: horseman });
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(other),
            },
        );
        choose_target(&mut game, Target::Card(own));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(own).zone, Zone::Hand);
        assert_eq!(game.object_view(other).zone, Zone::Graveyard);
        assert_eq!(game.object_view(horseman).zone, Zone::Exile);
        cast_adventurer(&mut game, horseman);
        settle(&mut game);
        assert_eq!(game.object_view(horseman).zone, Zone::Battlefield);

        cast(&mut game, murder, Some(Target::Card(horseman)));
        pass_twice(&mut game);
        assert_eq!(game.object_view(horseman).zone, Zone::Graveyard);
        let dead = game.object_ref(horseman);
        let mut stale = game.clone();
        stale.move_object(horseman, Zone::Exile);
        stale.move_object(horseman, Zone::Graveyard);
        assert_ne!(stale.object_ref(horseman), dead);
        settle(&mut stale);
        assert_eq!(stale.object_view(horseman).zone, Zone::Graveyard);

        settle(&mut game);
        assert_eq!(game.object_view(horseman).zone, Zone::Library);
        assert_eq!(
            game.state.players[player.index()].library.first(),
            Some(&horseman)
        );
    }
}

#[test]
fn gumdrop_poisoner_creates_food_and_uses_life_gained_not_net_life_change() {
    for player in [Player::P0, Player::P1] {
        let mut game = scenario(player);
        let poisoner = candidate(&mut game, player, "Gumdrop Poisoner // Tempt with Treats");
        let target = put(
            &mut game,
            player.other(),
            "Gigantosaurus",
            Zone::Battlefield,
        );
        let burn = put(&mut game, player.other(), "Shock", Zone::Hand);
        cast_adventure(&mut game, poisoner, None);
        settle(&mut game);
        assert_eq!(game.object_view(poisoner).zone, Zone::Exile);
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
        act(&mut game, Action::Pass);
        cast(&mut game, burn, Some(Target::Player(player)));
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 21);

        cast_adventurer(&mut game, poisoner);
        pass_twice(&mut game);
        assert!(game
            .object_view(poisoner)
            .keywords
            .contains(Keywords::LIFELINK));
        let mut no_target = game.clone();
        act(&mut no_target, Action::FinishTargets);
        settle(&mut no_target);
        assert_eq!(no_target.object_view(target).power, 10);

        choose_target(&mut game, Target::Card(target));
        settle(&mut game);
        let target = game.object_view(target);
        assert_eq!((target.power, target.toughness), (7, 7));
    }
}

#[test]
fn heartflame_duelist_grants_lifelink_to_a_paid_adventure_while_it_is_present() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let duelist = candidate(&mut game, player, "Heartflame Duelist // Heartflame Slash");
        let slash = candidate(&mut game, player, "Heartflame Duelist // Heartflame Slash");
        let bounce = put(&mut game, opponent, "Unsummon", Zone::Hand);
        announce(&mut game, duelist, &[]);
        settle(&mut game);
        cast_adventure(&mut game, slash, Some(Target::Player(opponent)));
        assert!(game
            .object_view(slash)
            .keywords
            .contains(Keywords::LIFELINK));
        let mut removed = game.clone();
        act(&mut removed, Action::Pass);
        cast(&mut removed, bounce, Some(Target::Card(duelist)));
        settle(&mut removed);
        assert_eq!(removed.state.players[player.index()].life, 20);
        assert_eq!(removed.state.players[opponent.index()].life, 17);
        assert_eq!(removed.object_view(slash).zone, Zone::Exile);

        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 23);
        assert_eq!(game.state.players[opponent.index()].life, 17);
        assert_eq!(game.object_view(slash).zone, Zone::Exile);
        cast_adventurer(&mut game, slash);
        settle(&mut game);
        assert_eq!(game.object_view(slash).zone, Zone::Battlefield);
    }
}

#[test]
fn horned_loch_whale_has_flash_off_turn_tapped_entry_and_ward_two() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut own_turn = scenario(player);
        let whale = candidate(&mut own_turn, player, "Horned Loch-Whale // Lagoon Breach");
        announce(&mut own_turn, whale, &[]);
        settle(&mut own_turn);
        assert!(!own_turn.object_view(whale).tapped);

        let mut game = scenario(opponent);
        let whale = candidate(&mut game, player, "Horned Loch-Whale // Lagoon Breach");
        let kill = put(&mut game, opponent, "Murder", Zone::Hand);
        act(&mut game, Action::Pass);
        announce(&mut game, whale, &[]);
        settle(&mut game);
        assert!(game.object_view(whale).tapped);
        assert!(game.object_view(whale).keywords.contains(Keywords::FLASH));

        cast(&mut game, kill, Some(Target::Card(whale)));
        pass_twice(&mut game);
        let mut paid = game.clone();
        reject(
            &mut paid,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 1],
            },
        );
        act(
            &mut paid,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 2],
            },
        );
        settle(&mut paid);
        assert_eq!(paid.object_view(whale).zone, Zone::Graveyard);

        act(&mut game, Action::DeclineResolutionMana);
        settle(&mut game);
        assert_eq!(game.object_view(kill).zone, Zone::Graveyard);
        assert_eq!(game.object_view(whale).zone, Zone::Battlefield);
    }
}

#[test]
fn lagoon_breach_uses_the_attacker_owner_for_the_private_library_choice() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(opponent);
        let card = candidate(&mut game, player, "Horned Loch-Whale // Lagoon Breach");
        let lands = [0; 2].map(|_| put(&mut game, player, "Island", Zone::Battlefield));
        let attacker = put(&mut game, player, "Harried Spearguard", Zone::Battlefield);
        let waiting = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        let exact = game.object_ref(attacker);
        game.change_controller(exact, opponent);
        pass_twice(&mut game);
        pass_twice(&mut game);
        act(&mut game, Action::Attack { card: attacker });
        act(&mut game, Action::FinishAttackers);
        act(&mut game, Action::Pass);
        for card in lands {
            act(&mut game, Action::TapMana { card });
        }
        act(&mut game, Action::AnnounceAdventure { card });
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(waiting),
            },
        );
        choose_target(&mut game, Target::Card(attacker));
        pay(&mut game);
        pass_twice(&mut game);
        assert_eq!(game.actor(), Some(player));
        assert!(game.observe(player).target_library.is_some());
        assert!(game.observe(opponent).target_library.is_none());
        let mut bottom = game.clone();
        for (branch, destination) in [
            (&mut game, LibraryDestination::Top),
            (&mut bottom, LibraryDestination::Bottom),
        ] {
            act(
                branch,
                Action::ChooseLibraryDestination {
                    object: exact,
                    destination,
                },
            );
            assert_eq!(branch.object_view(attacker).zone, Zone::Library);
            assert_eq!(branch.object_view(attacker).owner, player);
            assert_eq!(branch.object_view(card).zone, Zone::Exile);
            let library = &branch.state.players[player.index()].library;
            let selected_end = if destination == LibraryDestination::Top {
                library.last()
            } else {
                library.first()
            };
            assert_eq!(selected_end, Some(&attacker));
        }
    }
}

#[test]
fn birthright_boon_searches_both_types_and_kellan_counts_current_attachments() {
    const WICKED_ROLE: Card = Card::from_catalogue_id(5248);
    use rustmage_core::engine::replacements::TokenGroupView;

    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let kellan = candidate(
            &mut game,
            player,
            "Kellan, the Fae-Blooded // Birthright Boon",
        );
        let equipment = put(&mut game, player, "Swiftfoot Boots", Zone::Library);
        let aura = put(&mut game, player, "Bestial Bloodline", Zone::Library);
        let ordinary = put(&mut game, player, "Bear Cub", Zone::Library);
        let friend = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        let foreign = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        cast_adventure(&mut game, kellan, None);
        pass_twice(&mut game);
        assert!(game.observe(player).search_choice.is_some());
        assert!(game.observe(opponent).search_choice.is_none());
        reject(&mut game, Action::ChooseSearch { card: ordinary });
        let mut aura_branch = game.clone();
        for (branch, selection) in [(&mut game, equipment), (&mut aura_branch, aura)] {
            act(branch, Action::ChooseSearch { card: selection });
            act(branch, Action::FinishSearch);
            assert_eq!(branch.object_view(selection).zone, Zone::Hand);
            assert_eq!(branch.object_view(kellan).zone, Zone::Exile);
            assert!(branch
                .observe(opponent)
                .known_hand
                .contains(&branch.object_ref(selection)));
        }

        cast_adventurer(&mut game, kellan);
        settle(&mut game);
        assert!(game
            .object_view(kellan)
            .keywords
            .contains(Keywords::DOUBLE_STRIKE));
        announce(&mut game, equipment, &[]);
        settle(&mut game);
        let equip = Action::AnnounceActivate {
            card: equipment,
            ability: 0,
            counter_payment: Counters::default(),
        };
        act(&mut game, equip.clone());
        choose_target(&mut game, Target::Card(kellan));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(friend).power, 3);

        let recipient = game.object_ref(kellan);
        let _ = game.create_token_groups_attached(
            opponent,
            &[TokenGroupView {
                token: WICKED_ROLE,
                amount: 1,
                tapped: false,
            }],
            None,
            true,
            Some(recipient),
        );
        assert_eq!(game.object_view(friend).power, 4);
        assert_eq!(game.object_view(friend).toughness, 2);
        assert_eq!(game.object_view(foreign).power, 2);
        assert_eq!(game.object_view(kellan).power, 3);

        act(&mut game, equip);
        choose_target(&mut game, Target::Card(friend));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(friend).power, 3);
    }
}

#[test]
fn ego_drain_reveals_the_opposing_hand_and_privately_exiles_without_a_faerie() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let spell = candidate(&mut game, player, "Ego Drain");
        let own = put(&mut game, player, "Island", Zone::Hand);
        let land = put(&mut game, opponent, "Forest", Zone::Hand);
        let nonland = put(&mut game, opponent, "Bear Cub", Zone::Hand);
        put(&mut game, opponent, "Barrow Naughty", Zone::Battlefield);

        let mut only_land = game.clone();
        only_land.move_object(nonland, Zone::Graveyard);
        cast(&mut only_land, spell, Some(Target::Player(opponent)));
        pass_twice(&mut only_land);
        assert!(only_land
            .observe(player)
            .known_hand
            .contains(&only_land.object_ref(land)));
        assert!(only_land.observe(player).effect_choice.is_some());
        assert!(only_land.observe(opponent).effect_choice.is_none());
        let exile = Action::SelectResolutionExile {
            object: only_land.object_ref(own),
        };
        reject(&mut only_land, Action::DeclineEffectPayment);
        act(&mut only_land, exile.clone());
        assert_eq!(only_land.object_view(own).zone, Zone::Exile);
        assert_eq!(only_land.object_view(land).zone, Zone::Hand);

        let mut faerie = game.clone();
        put(&mut faerie, player, "Barrow Naughty", Zone::Battlefield);
        for branch in [&mut game, &mut faerie] {
            act(branch, Action::AnnounceCast { card: spell });
            reject(
                branch,
                Action::ChooseTarget {
                    target: Target::Player(player),
                },
            );
            choose_target(branch, Target::Player(opponent));
            pay(branch);
            pass_twice(branch);
            let land_ref = branch.object_ref(land);
            let nonland_ref = branch.object_ref(nonland);
            let known = branch.observe(player).known_hand;
            assert!(known.contains(&land_ref));
            assert!(known.contains(&nonland_ref));
            reject(branch, Action::ChooseRevealedDiscard { object: land_ref });
            act(
                branch,
                Action::ChooseRevealedDiscard {
                    object: nonland_ref,
                },
            );
            assert_eq!(branch.object_view(nonland).zone, Zone::Graveyard);
        }
        assert_eq!(faerie.object_view(own).zone, Zone::Hand);
        assert_eq!(faerie.object_view(spell).zone, Zone::Graveyard);
        assert!(game.observe(player).effect_choice.is_some());
        assert!(game.observe(opponent).effect_choice.is_none());
        act(&mut game, exile);
        assert_eq!(game.object_view(own).zone, Zone::Exile);
    }
}

#[test]
fn tempting_apple_steals_and_untaps_then_pays_for_either_food_ability() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let apple = candidate(&mut game, player, "Eriette's Tempting Apple");
        let creature = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        game.state.objects[creature as usize].tapped = true;
        announce(&mut game, apple, &[]);
        pass_twice(&mut game);
        choose_target(&mut game, Target::Card(creature));
        settle(&mut game);
        let stolen = game.object_view(creature);
        assert_eq!(stolen.controller, player);
        assert!(!stolen.tapped);
        assert!(stolen.keywords.contains(Keywords::HASTE));
        assert!(game.has_subtype(apple, "Food"));

        let mut gain = game.clone();
        act(
            &mut gain,
            Action::AnnounceActivate {
                card: apple,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut gain);
        assert_eq!(gain.object_view(apple).zone, Zone::Graveyard);
        settle(&mut gain);
        assert_eq!(gain.state.players[player.index()].life, 23);
        assert_eq!(gain.state.players[opponent.index()].life, 20);

        act(
            &mut game,
            Action::AnnounceActivate {
                card: apple,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Player(player),
            },
        );
        choose_target(&mut game, Target::Player(opponent));
        pay(&mut game);
        assert_eq!(game.object_view(apple).zone, Zone::Graveyard);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].life, 20);
        assert_eq!(game.state.players[opponent.index()].life, 17);
        assert_eq!(game.object_view(creature).controller, player);
        game.state.step = Step::End;
        pass_twice(&mut game);
        let returned = game.object_view(creature);
        assert_eq!(returned.controller, opponent);
        assert!(!returned.keywords.contains(Keywords::HASTE));
    }
}

#[test]
fn greta_makes_food_and_pays_the_right_food_for_each_activation_timing() {
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let greta = candidate(&mut game, player, "Greta, Sweettooth Scourge");
        let artifact = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
        let nontoken = put(&mut game, player, "Gingerbrute", Zone::Battlefield);
        let foreign = spawn(&mut game, opponent, FOOD, Zone::Battlefield);
        let burn = put(&mut game, opponent, "Shock", Zone::Hand);
        announce(&mut game, greta, &[]);
        settle(&mut game);
        let food = tokens(&game, player, FOOD);
        assert_eq!(food.len(), 1);
        let activate = |ability| Action::AnnounceActivate {
            card: greta,
            ability,
            counter_payment: Counters::default(),
        };

        act(&mut game, activate(0));
        choose_target(&mut game, Target::Card(greta));
        for card in [artifact, foreign] {
            let object = game.object_ref(card);
            reject(&mut game, Action::ChooseAdditionalCost { object });
        }
        let object = game.object_ref(food[0]);
        act(&mut game, Action::ChooseAdditionalCost { object });
        pay(&mut game);
        assert_ne!(game.object_view(food[0]).zone, Zone::Battlefield);
        settle(&mut game);
        assert_eq!(game.counter_count(greta, CounterKind::PlusOne), 1);

        act(&mut game, Action::Pass);
        cast(&mut game, burn, Some(Target::Player(opponent)));
        act(&mut game, Action::Pass);
        reject(&mut game, activate(0));
        act(&mut game, activate(1));
        let object = game.object_ref(nontoken);
        act(&mut game, Action::ChooseAdditionalCost { object });
        pay(&mut game);
        assert_eq!(game.object_view(nontoken).zone, Zone::Graveyard);
        pass_twice(&mut game);
        assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
        assert_eq!(game.state.players[player.index()].life, 19);
        assert_eq!(game.object_view(burn).zone, Zone::Stack);
        settle(&mut game);
        assert_eq!(game.state.players[opponent.index()].life, 18);
    }
}

#[test]
fn lord_skitter_counts_each_other_rat_and_creates_a_nonblocking_rat_at_combat() {
    const RAT: Card = Card::from_catalogue_id(5323);
    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let lord = candidate(&mut game, player, "Lord Skitter, Sewer King");
        let song = put(&mut game, player, "Song of Totentanz", Zone::Hand);
        let own = put(&mut game, player, "Forest", Zone::Graveyard);
        let first = put(&mut game, opponent, "Island", Zone::Graveyard);
        let second = put(&mut game, opponent, "Bear Cub", Zone::Graveyard);
        let declined = put(&mut game, opponent, "Swamp", Zone::Graveyard);

        announce(&mut game, lord, &[]);
        settle(&mut game);
        assert!(tokens(&game, player, RAT).is_empty());
        act(&mut game, Action::AnnounceCast { card: song });
        act(&mut game, Action::ChooseX { value: 2 });
        pay(&mut game);
        pass_twice(&mut game);
        assert_eq!(tokens(&game, player, RAT).len(), 2);

        for victim in [first, second] {
            if matches!(game.state.pending, Pending::OrderTriggers { .. }) {
                choose(&mut game, |action| {
                    matches!(action, Action::PutTrigger { .. })
                });
            }
            reject(
                &mut game,
                Action::ChooseTarget {
                    target: Target::Card(own),
                },
            );
            choose_target(&mut game, Target::Card(victim));
        }
        settle(&mut game);
        assert_eq!(game.object_view(first).zone, Zone::Exile);
        assert_eq!(game.object_view(second).zone, Zone::Exile);
        assert_eq!(game.object_view(own).zone, Zone::Graveyard);

        pass_twice(&mut game);
        pass_twice(&mut game);
        act(&mut game, Action::FinishTargets);
        settle(&mut game);
        let rats = tokens(&game, player, RAT);
        assert_eq!(rats.len(), 3);
        for rat in rats {
            let rat = game.object_view(rat);
            assert_eq!((rat.power, rat.toughness), (1, 1));
            assert!(rat.keywords.contains(Keywords::CANT_BLOCK));
        }
        assert_eq!(game.object_view(declined).zone, Zone::Graveyard);
    }
}

#[test]
fn hopeless_nightmare_discards_and_loses_life_then_scries_after_paid_sacrifice() {
    let card = Card::from_name("Hopeless Nightmare").unwrap();
    let mut deck = vec![Card::Forest; 59];
    deck.push(card);
    let mut config = Config {
        decks: [deck, vec![Card::Forest; 60]],
        seed: 20261023,
        starting_player: Player::P0,
        deck_rules: DeckRules::Standard,
    };
    assert!(
        matches!(Game::new(config.clone()), Err(Error::InvalidDeck(message))
        if message == "not Standard legal in this catalogue: Hopeless Nightmare")
    );
    config.deck_rules = DeckRules::Freeform;
    assert_eq!(Game::new(config).is_ok(), card.is_supported());

    for player in [Player::P0, Player::P1] {
        let opponent = player.other();
        let mut game = scenario(player);
        let nightmare = candidate(&mut game, player, "Hopeless Nightmare");
        let discard = put(&mut game, opponent, "Bear Cub", Zone::Hand);
        let keep = put(&mut game, player, "Island", Zone::Library);
        let bottom = put(&mut game, player, "Swamp", Zone::Library);

        let mut empty = game.clone();
        empty.move_object(discard, Zone::Library);
        announce(&mut empty, nightmare, &[]);
        settle(&mut empty);
        assert_eq!(empty.state.players[opponent.index()].life, 18);

        announce(&mut game, nightmare, &[]);
        pass_twice(&mut game);
        assert_eq!(game.actor(), Some(opponent));
        act(&mut game, Action::Discard { card: discard });
        settle(&mut game);
        assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
        assert_eq!(game.state.players[opponent.index()].life, 18);

        act(
            &mut game,
            Action::AnnounceActivate {
                card: nightmare,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut game);
        assert_eq!(game.object_view(nightmare).zone, Zone::Battlefield);
        pass_twice(&mut game);
        assert_eq!(game.object_view(nightmare).zone, Zone::Graveyard);
        assert!(game.observe(player).library_choice.is_none());
        pass_twice(&mut game);
        assert_eq!(
            game.observe(player).library_choice.unwrap().remaining.len(),
            2
        );
        assert!(game.observe(opponent).library_choice.is_none());
        act(
            &mut game,
            Action::ArrangeLibrary {
                card: bottom,
                destination: LibraryDestination::Bottom,
            },
        );
        act(
            &mut game,
            Action::ArrangeLibrary {
                card: keep,
                destination: LibraryDestination::Top,
            },
        );
        settle(&mut game);
        assert_eq!(game.top_library_card(player), Some(keep));
        assert_eq!(
            game.state.players[player.index()].library.first(),
            Some(&bottom)
        );
    }
}
