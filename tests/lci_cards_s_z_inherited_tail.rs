//! Real-card obligations for inherited LCI cards from Synapse through Waylaying.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const GNOME: Card = Card(5267);
const MAP: Card = Card(5284);

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn announce(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn attack(game: &mut Game, card: CardId) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    act(game, Action::Attack { card });
    act(game, Action::FinishAttackers);
}

fn token_ids(game: &Game, p: Player, card: Card) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.card == card && object.zone == Zone::Battlefield && object.controller == p)
                .then_some(id as CardId)
        })
        .collect()
}

fn pay_if_pending(game: &mut Game) {
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn sacrifice_as_cost(game: &mut Game, sacrifice: CardId, invalid: CardId) {
    pay_if_pending(game);
    let wrong = Action::ChooseAdditionalCost {
        object: game.object_ref(invalid),
    };
    reject(game, wrong);
    let object = game.object_ref(sacrifice);
    act(game, Action::ChooseAdditionalCost { object });
    pay_if_pending(game);
    assert_eq!(game.object_view(sacrifice).zone, Zone::Graveyard);
}

#[test]
fn tectonic_hazard_damages_only_its_opponent_and_their_creatures_and_synapse_tokens_need_death() {
    let p = Player::P0;
    let opponent = p.other();
    for finality in [false, true] {
        let mut game = scenario(p);
        let own = put(&mut game, p, "Bear Cub", Zone::Battlefield);
        let foe = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
        let necromage = put(&mut game, opponent, "Synapse Necromage", Zone::Battlefield);
        let spell = put(&mut game, p, "Tectonic Hazard", Zone::Hand);
        if finality {
            game.add_counter(necromage, CounterKind::Finality, 1);
        }
        let first_possible_token = game.state.objects.len();
        cast(&mut game, spell, None);
        settle(&mut game);
        assert_eq!(game.state.players[p.index()].life, STARTING_LIFE);
        assert_eq!(game.state.players[opponent.index()].life, STARTING_LIFE - 1);
        assert_eq!(game.object_view(own).damage, 0);
        assert_eq!(game.object_view(foe).damage, 1);
        assert_eq!(
            game.object_view(necromage).zone,
            if finality {
                Zone::Exile
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(
            game.state.objects.len() - first_possible_token,
            if finality { 0 } else { 2 }
        );
        for id in first_possible_token..game.state.objects.len() {
            let id = id as CardId;
            let token = game.object_view(id);
            assert!(token.is_token() && token.has_subtype("Fungus"));
            assert_eq!(token.controller, opponent);
            assert_eq!(token.colors, 4);
            assert_eq!(stats(&game, id), (1, 1));
            assert_eq!(token.damage, 0);
            assert!(game.keywords(id).contains(Keywords::CANT_BLOCK));
        }
    }
}

#[test]
fn thrashing_brontodon_sacrifices_itself_to_destroy_either_artifact_or_enchantment() {
    let p = Player::P1;
    let mut game = scenario(p);
    let brontodon = put(&mut game, p, "Thrashing Brontodon", Zone::Battlefield);
    let artifact = put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    let enchantment = put(&mut game, p.other(), "Impact Tremors", Zone::Battlefield);
    let creature = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    for candidate in [artifact, enchantment] {
        let mut branch = game.clone();
        act(&mut branch, announce(brontodon));
        reject(&mut branch, target(creature));
        act(&mut branch, target(candidate));
        pay(&mut branch);
        assert_eq!(branch.object_view(brontodon).zone, Zone::Graveyard);
        let mut protected = branch.clone();
        protected.grant_keyword_effects(
            &[candidate],
            protected.object_ref(candidate),
            Keywords::INDESTRUCTIBLE,
        );
        settle(&mut protected);
        assert_eq!(protected.object_view(candidate).zone, Zone::Battlefield);
        settle(&mut branch);
        assert_eq!(branch.object_view(candidate).zone, Zone::Graveyard);
    }
}

#[test]
fn tinkers_tote_creates_two_gnomes_and_its_sacrifice_is_a_cost_even_when_countered() {
    let p = Player::P0;
    let mut game = scenario(p);
    let tote = put(&mut game, p, "Tinker's Tote", Zone::Hand);
    let binder = put(&mut game, p.other(), "Tishana's Tidebinder", Zone::Hand);
    cast(&mut game, tote, None);
    settle(&mut game);
    let gnomes = token_ids(&game, p, GNOME);
    assert_eq!(gnomes.len(), 2);
    for card in gnomes {
        let view = game.object_view(card);
        assert!(view.is_token() && view.has_subtype("Gnome"));
        assert_eq!(view.kinds, 1 | 16);
        assert_eq!(view.colors, 0);
        assert_eq!(stats(&game, card), (1, 1));
    }
    game.state.players[p.index()].mana = [1, 0, 0, 0, 0, 0];
    act(&mut game, announce(tote));
    pay(&mut game);
    assert_eq!(game.object_view(tote).zone, Zone::Graveyard);
    let mut success = game.clone();
    settle(&mut success);
    assert_eq!(success.state.players[p.index()].life, STARTING_LIFE + 3);
    let ability = game.state.stack.last().unwrap().id;
    act(&mut game, Action::Pass);
    cast(&mut game, binder, None);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Stack(ability),
        },
    );
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].life, STARTING_LIFE);
    assert_eq!(game.object_view(tote).zone, Zone::Graveyard);
}

#[test]
fn tidebinder_blanks_the_exact_activated_source_and_cannot_target_a_spell_or_mana_ability() {
    let p = Player::P0;
    let opponent = p.other();
    let mut game = scenario(p);
    let envoy = put(&mut game, p, "Envoy of Okinec Ahau", Zone::Battlefield);
    let forest = put(&mut game, p, "Forest", Zone::Battlefield);
    let spell = put(&mut game, p, "Shock", Zone::Hand);
    let binder = put(&mut game, opponent, "Tishana's Tidebinder", Zone::Hand);
    let target_ability = |id| Action::ChooseTarget {
        target: Target::Stack(id),
    };
    let mut no_ability = game.clone();
    act(&mut no_ability, Action::TapMana { card: forest });
    cast(&mut no_ability, spell, Some(Target::Player(opponent)));
    let id = no_ability.state.stack.last().unwrap().id;
    act(&mut no_ability, Action::Pass);
    cast(&mut no_ability, binder, None);
    pass_pair(&mut no_ability);
    reject(&mut no_ability, target_ability(id));
    if no_ability.legal_actions().contains(&Action::FinishTargets) {
        act(&mut no_ability, Action::FinishTargets);
    }
    settle(&mut no_ability);
    assert_eq!(
        no_ability.state.players[opponent.index()].life,
        STARTING_LIFE - 2
    );

    game.state.players[p.index()].mana = [1, 0, 0, 0, 0, 4];
    act(&mut game, announce(envoy));
    pay(&mut game);
    let id = game.state.stack.last().unwrap().id;
    act(&mut game, Action::Pass);
    cast(&mut game, binder, None);
    pass_pair(&mut game);
    act(&mut game, target_ability(id));
    settle(&mut game);
    assert!(token_ids(&game, p, GNOME).is_empty());
    assert!(!game.printed_abilities_available(envoy));
    let original = game.object_ref(binder);
    game.move_object(binder, Zone::Hand);
    game.move_object(binder, Zone::Battlefield);
    assert_ne!(game.object_ref(binder), original);
    assert!(game.printed_abilities_available(envoy));
}

#[test]
fn unlucky_drop_lets_the_stolen_permanents_owner_choose_top_or_bottom() {
    let p = Player::P1;
    let opponent = p.other();
    let mut game = scenario(p);
    let victim = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let land = put(&mut game, p, "Forest", Zone::Battlefield);
    let spell = put(&mut game, p, "Unlucky Drop", Zone::Hand);
    game.record_control(game.object_ref(victim), p, None);
    let object = game.object_ref(victim);
    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(land));
    act(&mut game, target(victim));
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    for destination in [LibraryDestination::Top, LibraryDestination::Bottom] {
        let mut branch = game.clone();
        let action = Action::ChooseLibraryDestination {
            object,
            destination,
        };
        assert_eq!(branch.apply(p, action.clone()), Err(Error::WrongActor));
        act(&mut branch, action);
        let library = &branch.state.players[opponent.index()].library;
        let expected = if destination == LibraryDestination::Top {
            library.last()
        } else {
            library.first()
        };
        assert_eq!(expected, Some(&victim));
        assert_eq!(branch.object_view(victim).zone, Zone::Library);
    }
}

#[test]
fn vanguard_sacrifices_another_permanent_to_survive_damage_then_loses_indestructible() {
    let p = Player::P0;
    let mut game = scenario(p);
    let vanguard = put(&mut game, p, "Vanguard of the Rose", Zone::Battlefield);
    let artifact = put(&mut game, p, "Short Bow", Zone::Battlefield);
    let shock = put(&mut game, p.other(), "Shock", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, shock, Some(Target::Card(vanguard)));
    act(&mut game, Action::Pass);
    act(&mut game, announce(vanguard));
    sacrifice_as_cost(&mut game, artifact, vanguard);
    settle(&mut game);
    assert_eq!(game.object_view(vanguard).zone, Zone::Battlefield);
    assert_eq!(game.object_view(vanguard).damage, 2);
    assert!(game.object_view(vanguard).tapped);
    assert!(game.keywords(vanguard).contains(Keywords::INDESTRUCTIBLE));
    game.state.step = Step::End;
    pass_pair(&mut game);
    assert!(!game.keywords(vanguard).contains(Keywords::INDESTRUCTIBLE));
    assert_eq!(game.object_view(vanguard).damage, 0);
}

#[test]
fn vitos_inquisitor_keeps_the_counter_but_loses_menace_at_cleanup() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Vito's Inquisitor", Zone::Battlefield);
    let victim = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    game.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
    act(&mut game, announce(source));
    sacrifice_as_cost(&mut game, victim, source);
    settle(&mut game);
    assert_eq!(stats(&game, source), (4, 4));
    assert!(game.keywords(source).contains(Keywords::MENACE));
    game.state.step = Step::End;
    pass_pair(&mut game);
    assert!(!game.keywords(source).contains(Keywords::MENACE));
    assert_eq!(game.counter_count(source, CounterKind::PlusOne), 1);
}

#[test]
fn volatile_wanderglyph_offers_the_discard_when_tapped_by_attack_or_crew() {
    let p = Player::P0;
    for crew in [false, true] {
        let mut game = scenario(p);
        let card = put(&mut game, p, "Volatile Wanderglyph", Zone::Battlefield);
        let discard = put(&mut game, p, "Opt", Zone::Hand);
        let top = put(&mut game, p, "Island", Zone::Library);
        if crew {
            let vehicle = put(&mut game, p, "Dune Drifter", Zone::Battlefield);
            act(&mut game, announce(vehicle));
            pay_if_pending(&mut game);
            let object = game.object_ref(card);
            act(&mut game, Action::ChooseCrewer { object });
            act(&mut game, Action::FinishCrew);
        } else {
            attack(&mut game, card);
        }
        for _ in 0..10 {
            if game
                .legal_actions()
                .contains(&Action::Discard { card: discard })
            {
                break;
            }
            choose(&mut game, |action| {
                matches!(action, Action::Pass | Action::PutTrigger { .. })
            });
        }
        assert!(game.object_view(card).tapped);
        let mut declined = game.clone();
        act(&mut declined, Action::DeclineEffectPayment);
        settle(&mut declined);
        assert_eq!(declined.object_view(discard).zone, Zone::Hand);
        assert_eq!(declined.object_view(top).zone, Zone::Library);
        act(&mut game, Action::Discard { card: discard });
        settle(&mut game);
        assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
        assert_eq!(game.object_view(top).zone, Zone::Hand);
    }
}

#[test]
fn waterwind_scout_flies_and_its_map_explores_after_paying_and_sacrificing() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Waterwind Scout", Zone::Hand);
    let top = put(&mut game, p, "Island", Zone::Library);
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(game.keywords(source).contains(Keywords::FLYING));
    let maps = token_ids(&game, p, MAP);
    assert_eq!(maps.len(), 1);
    let map = maps[0];
    game.state.step = Step::Upkeep;
    reject(&mut game, announce(map));
    game.state.step = Step::Main1;
    act(&mut game, announce(map));
    act(&mut game, target(source));
    pay(&mut game);
    settle(&mut game);
    assert!(!game.on_battlefield(map));
    assert_eq!(game.object_view(top).zone, Zone::Hand);
}

#[test]
fn waylaying_pirates_checks_for_an_artifact_twice_and_stuns_only_an_opposing_target() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Waylaying Pirates", Zone::Hand);
    let target_creature = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    let own = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let mut no_artifact = game.clone();
    cast(&mut no_artifact, source, None);
    settle(&mut no_artifact);
    assert!(!no_artifact.object_view(target_creature).tapped);

    let artifact = put(&mut game, p, "Short Bow", Zone::Battlefield);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    reject(&mut game, target(own));
    act(&mut game, target(target_creature));
    let mut removed = game.clone();
    removed.move_object(artifact, Zone::Hand);
    settle(&mut removed);
    assert_eq!(removed.counter_count(target_creature, CounterKind::Stun), 0);
    assert!(!removed.object_view(target_creature).tapped);

    settle(&mut game);
    assert!(game.object_view(target_creature).tapped);
    assert_eq!(game.counter_count(target_creature, CounterKind::Stun), 1);
    game.untap_objects(&[target_creature], untap_rules::UntapCause::Effect);
    assert!(game.object_view(target_creature).tapped);
    assert_eq!(game.counter_count(target_creature, CounterKind::Stun), 0);
    game.untap_objects(&[target_creature], untap_rules::UntapCause::Effect);
    assert!(!game.object_view(target_creature).tapped);
}

#[test]
fn enigma_jewel_funds_abilities_and_locus_uses_and_copies_each_borrowed_slot_separately() {
    let p = Player::P0;
    let mut game = scenario(p);
    let jewel = put(&mut game, p, "The Enigma Jewel", Zone::Hand);
    let boots = put(&mut game, p, "Swiftfoot Boots", Zone::Battlefield);
    let creature = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut game, p, "Candy Trail", Zone::Hand);
    let forest = put(&mut game, p, "Forest", Zone::Graveyard);
    let first = put(&mut game, p, "Envoy of Okinec Ahau", Zone::Graveyard);
    let second = put(&mut game, p, "Envoy of Okinec Ahau", Zone::Graveyard);
    let elf = put(&mut game, p, "Llanowar Elves", Zone::Graveyard);
    let fourth = put(&mut game, p, "Hermitic Nautilus", Zone::Battlefield);
    game.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
    cast(&mut game, jewel, None);
    settle(&mut game);
    assert!(game.object_view(jewel).tapped);
    let generate = Action::ActivateMana {
        card: jewel,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: [0, 0, 0, 0, 0, 2],
    };
    reject(&mut game, generate.clone());
    game.untap_objects(&[jewel], untap_rules::UntapCause::Effect);
    act(&mut game, generate);
    reject(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, announce(boots));
    act(&mut game, target(creature));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(
        game.object_view(boots).attached_to,
        Some(game.object_ref(creature))
    );
    assert_eq!(game.state.players[p.index()].mana, [0, 0, 0, 0, 0, 1]);

    game.clear_step_mana(false, false);
    game.state.players[p.index()].mana = [0, 1, 0, 0, 0, 8];
    act(
        &mut game,
        Action::AnnounceActivate {
            card: jewel,
            ability: 1,
            counter_payment: Counters::default(),
        },
    );
    act(&mut game, Action::BeginOrderedCosts);
    let clause = game
        .observe(p)
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::Additional)
        .unwrap() as u16;
    act(&mut game, Action::ChooseCastCost { clause });
    for card in [forest, creature] {
        let invalid = Action::ChooseAdditionalCost {
            object: game.object_ref(card),
        };
        reject(&mut game, invalid);
    }
    for card in [first, second, elf, fourth] {
        let object = game.object_ref(card);
        act(&mut game, Action::ChooseAdditionalCost { object });
    }
    act(&mut game, Action::FinishCraftMaterials);
    for _ in 0..8 {
        if game.state.announcing.is_none() {
            break;
        }
        choose(&mut game, |action| {
            matches!(
                action,
                Action::PayCosts { .. } | Action::ChooseCastCost { .. }
            )
        });
    }
    settle(&mut game);
    assert_eq!(game.object_view(jewel).name, "Locus of Enlightenment");
    assert_eq!(game.observe(p).borrowed_abilities.len(), 4);

    let mut mana = game.clone();
    let exiled = mana.object_ref(elf);
    let mana_slot = mana
        .observe(p)
        .borrowed_abilities
        .into_iter()
        .find(|ability| ability.exiled == exiled)
        .unwrap();
    act(
        &mut mana,
        Action::AnnounceBorrowedMana {
            card: jewel,
            exiled,
            ability: mana_slot.ability,
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 1, 0],
        },
    );
    pay_if_pending(&mut mana);
    assert!(mana.state.stack.is_empty());
    assert_eq!(mana.state.players[p.index()].mana, [0, 0, 0, 0, 1, 0]);

    game.state.players[p.index()].mana = [2, 0, 0, 0, 0, 8];
    let borrow = |game: &Game, donor| Action::AnnounceBorrowedAbility {
        card: jewel,
        exiled: game.object_ref(donor),
        ability: 0,
        counter_payment: Counters::default(),
    };
    for (donor, expected) in [(first, 2), (second, 4)] {
        let activation = borrow(&game, donor);
        act(&mut game, activation.clone());
        act(
            &mut game,
            Action::PayCosts {
                payment: [1, 0, 0, 0, 0, 4],
            },
        );
        settle(&mut game);
        assert_eq!(token_ids(&game, p, GNOME).len(), expected);
        reject(&mut game, activation);
    }
    game.move_object(first, Zone::Graveyard);
    assert_eq!(game.observe(p).borrowed_abilities.len(), 3);
}
