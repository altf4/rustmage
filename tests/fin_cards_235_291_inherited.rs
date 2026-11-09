//! Native outcomes for the inherited Final Fantasy definitions in rows 235–291.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn passes(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn next_chapter(game: &mut Game) {
    game.state.step = Step::Draw;
    passes(game);
    assert_eq!(game.state.step, Step::Main1);
}

fn targets(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn modal(game: &mut Game, card: CardId, mode: u16, target: Target) {
    act(game, Action::AnnounceCast { card });
    act(game, Action::ChooseMode { mode });
    targets(game, target);
}

fn announce(game: &mut Game, card: CardId, ability: u16) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
}

fn reach(game: &mut Game, action: &Action) {
    for _ in 0..12 {
        if game.legal_actions().contains(action) {
            return;
        }
        choose(game, |candidate| {
            matches!(candidate, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("action {action:?} was not reached");
}

fn attack(game: &mut Game, attackers: &[CardId]) {
    for _ in 0..8 {
        if game.state.pending == Pending::Attackers {
            break;
        }
        act(game, Action::Pass);
    }
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn tokens(game: &Game, card: Card) -> Vec<CardId> {
    game.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, object)| {
            (object.zone == Zone::Battlefield && object.is_token() && object.card == card)
                .then_some(id as CardId)
        })
        .collect()
}

#[test]
fn ramuh_counts_only_noncreature_nonlands_and_the_later_chapters_buff_wizards() {
    let player = Player::P0;
    let mut game = scenario(player);
    for name in ["Shock", "Swiftfoot Boots", "Island", "Bear Cub"] {
        put(&mut game, player, name, Zone::Graveyard);
    }
    let ally = put(
        &mut game,
        player,
        "Shantotto, Tactician Magician",
        Zone::Battlefield,
    );
    let bear = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let opposing = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let enemy_wizard = put(
        &mut game,
        player.other(),
        "Shantotto, Tactician Magician",
        Zone::Battlefield,
    );
    let ramuh = put(&mut game, player, "Summon: Esper Ramuh", Zone::Hand);
    cast(&mut game, ramuh, None);
    passes(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    targets(&mut game, Target::Card(opposing));
    settle(&mut game);
    assert_eq!(game.object_view(opposing).damage, 2);
    for power in [1, 2] {
        next_chapter(&mut game);
        settle(&mut game);
        assert_eq!(stats(&game, ally), (power, 4));
        assert_eq!(stats(&game, bear), (2, 2));
        assert_eq!(stats(&game, enemy_wizard), (0, 4));
    }
    assert_eq!(game.object_view(ramuh).zone, Zone::Graveyard);
}

#[test]
fn fat_chocobo_makes_a_landfall_bird_and_renews_trample_for_later_creatures() {
    let player = Player::P0;
    let mut game = scenario(player);
    let saga = put(&mut game, player, "Summon: Fat Chocobo", Zone::Hand);
    let opponent = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    cast(&mut game, saga, None);
    settle(&mut game);
    let bird = tokens(&game, Card(5269))[0];
    assert_eq!(stats(&game, bird), (2, 2));
    assert!(game.has_subtype(bird, "Bird"));
    assert_eq!(game.object_view(bird).colors, 16);
    let land = put(&mut game, player, "Forest", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    settle(&mut game);
    assert_eq!(stats(&game, bird), (3, 2));

    next_chapter(&mut game);
    settle(&mut game);
    assert!(game.keywords(bird).contains(Keywords::TRAMPLE));
    let later = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    assert!(!game.keywords(later).contains(Keywords::TRAMPLE));
    next_chapter(&mut game);
    settle(&mut game);
    assert!(game.keywords(later).contains(Keywords::TRAMPLE));
    let final_later = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    next_chapter(&mut game);
    settle(&mut game);
    assert!(game.keywords(final_later).contains(Keywords::TRAMPLE));
    assert!(!game.keywords(opponent).contains(Keywords::TRAMPLE));
    assert_eq!(game.object_view(saga).zone, Zone::Graveyard);
}

#[test]
fn ifrit_offers_two_private_discard_payments_then_two_red_mana_chapters() {
    let player = Player::P0;
    let mut game = scenario(player);
    let card = put(&mut game, player, "Island", Zone::Hand);
    let ifrit = put(&mut game, player, "Summon: G.F. Ifrit", Zone::Hand);
    cast(&mut game, ifrit, None);
    let decline = Action::DeclineEffectPayment;
    reach(&mut game, &decline);
    act(&mut game, decline);
    assert_eq!(game.object_view(card).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    next_chapter(&mut game);
    let discard = Action::Discard { card };
    reach(&mut game, &discard);
    act(&mut game, discard);
    assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
    for _ in 0..2 {
        next_chapter(&mut game);
        assert_eq!(game.state.stack.len(), 1);
        assert_eq!(game.state.players[player.index()].mana[3], 0);
        settle(&mut game);
        assert_eq!(game.state.players[player.index()].mana[3], 1);
    }
    assert_eq!(game.object_view(ifrit).zone, Zone::Graveyard);
}

#[test]
fn garuda_checks_a_tapped_enemy_then_grants_another_creature_flying_twice() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let victim = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let untouched = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    game.state.objects[victim as usize].tapped = true;
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let garuda = put(&mut game, player, "Summon: Primal Garuda", Zone::Hand);
    cast(&mut game, garuda, None);
    passes(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(untouched),
        },
    );
    targets(&mut game, Target::Card(victim));
    let mut untapped = game.clone();
    untapped.state.objects[victim as usize].tapped = false;
    settle(&mut untapped);
    assert_eq!(untapped.object_view(victim).zone, Zone::Battlefield);
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert!(game.keywords(garuda).contains(Keywords::FLYING));
    for power in [3, 4] {
        next_chapter(&mut game);
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(garuda),
            },
        );
        targets(&mut game, Target::Card(ally));
        settle(&mut game);
        assert_eq!(stats(&game, ally), (power, 2));
        assert!(game.keywords(ally).contains(Keywords::FLYING));
    }
    assert_eq!(game.object_view(garuda).zone, Zone::Graveyard);
}

#[test]
fn shiva_puts_stun_counters_even_on_an_already_tapped_creature_and_draws_for_current_ones() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let first = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut game, opponent, "Swiftfoot Boots", Zone::Battlefield);
    let friendly = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    for id in [second, artifact, friendly] {
        game.state.objects[id as usize].tapped = true;
    }
    let shiva = put(&mut game, player, "Summon: Shiva", Zone::Hand);
    cast(&mut game, shiva, None);
    passes(&mut game);
    for count in 1..=2 {
        if count == 2 {
            next_chapter(&mut game);
        }
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(friendly),
            },
        );
        targets(&mut game, Target::Card(first));
        settle(&mut game);
        assert!(game.object_view(first).tapped);
        assert_eq!(game.counter_count(first, CounterKind::Stun), count);
    }
    next_chapter(&mut game);
    let mut fewer = game.clone();
    fewer.move_object(second, Zone::Exile);
    settle(&mut fewer);
    assert_eq!(fewer.hand_count(player), 1);
    settle(&mut game);
    assert_eq!(game.hand_count(player), 2);
    assert_eq!(game.object_view(shiva).zone, Zone::Graveyard);
}

#[test]
fn summoners_grimoire_enters_with_a_shaman_and_only_an_enchantment_enters_attacking() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let equipment = put(&mut game, player, "Summoner's Grimoire", Zone::Hand);
    let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let normal = put(&mut game, player, "Hill Giant", Zone::Hand);
    let enchanted = put(&mut game, player, "Summon: G.F. Ifrit", Zone::Hand);
    cast(&mut game, equipment, None);
    settle(&mut game);
    let hero = tokens(&game, Card(5328))[0];
    assert!(game.has_subtype(hero, "Shaman"));
    assert_eq!(
        game.object_view(equipment).attached_to,
        Some(game.object_ref(hero))
    );
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 3];
    announce(&mut game, equipment, 0);
    targets(&mut game, Target::Card(attacker));
    pay(&mut game);
    settle(&mut game);
    assert!(game.has_subtype(attacker, "Shaman"));
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    attack(&mut game, &[attacker]);
    let put_normal = Action::PutHandCreature {
        object: game.object_ref(normal),
    };
    reach(&mut game, &put_normal);
    assert!(
        game.observe(opponent)
            .effect_choice
            .as_ref()
            .is_none_or(|view| view.candidates.is_empty())
    );
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineHandEntry);
    assert_eq!(declined.object_view(normal).zone, Zone::Hand);
    let mut regular = game.clone();
    act(&mut regular, put_normal);
    assert_eq!(regular.object_view(normal).zone, Zone::Battlefield);
    assert!(!regular.object_view(normal).tapped);
    assert!(
        !regular
            .state
            .attackers
            .iter()
            .any(|attack| attack.card == normal)
    );
    let object = game.object_ref(enchanted);
    act(&mut game, Action::PutHandCreature { object });
    let defender = Action::ChooseHandDefender {
        defender: CombatDefender::Player(opponent),
    };
    if game.legal_actions().contains(&defender) {
        act(&mut game, defender);
    }
    assert!(game.object_view(enchanted).tapped);
    assert_eq!(game.counter_count(enchanted, CounterKind::Lore), 1);
    assert!(
        game.state
            .attackers
            .iter()
            .any(|attack| attack.card == enchanted)
    );
    let ifrit_decline = Action::DeclineEffectPayment;
    reach(&mut game, &ifrit_decline);
    act(&mut game, ifrit_decline);
}

#[test]
fn suplex_tracks_death_even_after_prevention_and_can_exile_an_artifact_instead() {
    let player = Player::P0;
    let mut game = scenario(player);
    let creature = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let artifact = put(
        &mut game,
        player.other(),
        "Swiftfoot Boots",
        Zone::Battlefield,
    );
    game.add_counter(creature, CounterKind::Shield, 1);
    let suplex = put(&mut game, player, "Suplex", Zone::Hand);
    modal(&mut game, suplex, 0, Target::Card(creature));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(creature).damage, 0);
    assert_eq!(game.counter_count(creature, CounterKind::Shield), 0);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Exile);
    let suplex = put(&mut game, player, "Suplex", Zone::Hand);
    modal(&mut game, suplex, 1, Target::Card(artifact));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Exile);
}

#[test]
fn syncopate_taxes_the_spell_controller_and_exiles_only_an_actual_counter() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let spell = put(&mut game, opponent, "Shock", Zone::Hand);
    let counter = put(&mut game, player, "Syncopate", Zone::Hand);
    cast(&mut game, spell, Some(Target::Player(player)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::AnnounceCast { card: counter });
    act(&mut game, Action::ChooseX { value: 3 });
    targets(&mut game, Target::Spell(spell));
    pay(&mut game);
    passes(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    let request = game.observe(opponent).resolution_payment.unwrap();
    assert_eq!(request.cost.generic, 3);
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineResolutionMana);
    assert_eq!(decline.object_view(spell).zone, Zone::Exile);
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, 18);
}

#[test]
fn fire_crystal_discounts_red_grants_haste_and_its_temporary_copy_survives_the_source() {
    let player = Player::P0;
    let mut game = scenario(player);
    let fire = put(&mut game, player, "The Fire Crystal", Zone::Battlefield);
    let own = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let wizard = put(
        &mut game,
        player,
        "Shantotto, Tactician Magician",
        Zone::Hand,
    );
    act(&mut game, Action::AnnounceCast { card: wizard });
    let request = game.observe(player).payment_request.unwrap();
    assert_eq!(request.generic, 0);
    assert_eq!(request.required, [0, 1, 0, 1, 0, 0]);
    pay(&mut game);
    settle(&mut game);
    assert!(game.keywords(wizard).contains(Keywords::HASTE));
    assert!(!game.keywords(enemy).contains(Keywords::HASTE));
    game.state.players[player.index()].mana = [8; 6];
    let chosen = put(&mut game, player, "The Crystal's Chosen", Zone::Hand);
    cast(&mut game, chosen, None);
    settle(&mut game);
    let heroes = tokens(&game, Card(5328));
    assert_eq!(heroes.len(), 4);
    for hero in heroes {
        assert_eq!(game.object_view(hero).colors, 0);
        assert!(game.has_subtype(hero, "Hero"));
        assert_eq!(stats(&game, hero), (2, 2));
        assert!(game.keywords(hero).contains(Keywords::HASTE));
    }
    assert_eq!(game.counter_count(own, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(wizard, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(enemy, CounterKind::PlusOne), 0);
    game.state.players[player.index()].mana = [8; 6];
    announce(&mut game, fire, 0);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    targets(&mut game, Target::Card(own));
    let request = game.observe(player).payment_request.unwrap();
    assert_eq!(request.generic, 4);
    assert_eq!(request.required[3], 2);
    pay(&mut game);
    settle(&mut game);
    let copy = tokens(&game, game.object_view(own).card)[0];
    assert_eq!(stats(&game, copy), (2, 2));
    assert!(game.keywords(copy).contains(Keywords::HASTE));
    game.move_object(fire, Zone::Exile);
    assert!(game.object_view(copy).zone == Zone::Battlefield);
    game.state.step = Step::Main2;
    passes(&mut game);
    settle(&mut game);
    assert!(!game.on_battlefield(copy));
}

#[test]
fn minstrel_untaps_the_fifth_town_and_travel_pays_only_its_two_blue_symbols() {
    let player = Player::P0;
    let mut game = scenario(player);
    let minstrel = put(
        &mut game,
        player,
        "The Wandering Minstrel",
        Zone::Battlefield,
    );
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let mut old_town = None;
    for _ in 0..4 {
        old_town = Some(put(
            &mut game,
            player,
            "Vector, Imperial Capital",
            Zone::Battlefield,
        ));
    }
    let old_town = old_town.unwrap();
    game.state.objects[old_town as usize].tapped = true;
    put(&mut game, player, "Swamp", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Treno, Dark City",
        Zone::Battlefield,
    );
    let town = put(&mut game, player, "Treno, Dark City", Zone::Hand);
    let mut ordinary = game.clone();
    ordinary.move_object(minstrel, Zone::Exile);
    act(&mut ordinary, Action::PlayLand { card: town });
    assert!(ordinary.object_view(town).tapped);
    act(&mut game, Action::PlayLand { card: town });
    assert!(!game.object_view(town).tapped);
    let travel = put(&mut game, player, "Travel the Overworld", Zone::Hand);
    game.state.players[player.index()].mana = [0, 2, 0, 0, 0, 0];
    act(&mut game, Action::AnnounceCast { card: travel });
    let request = game.observe(player).payment_request.unwrap();
    assert_eq!(request.generic, 0);
    assert_eq!(request.required, [0, 2, 0, 0, 0, 0]);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.hand_count(player), 4);
    let mut short = game.clone();
    short.move_object(old_town, Zone::Exile);
    for branch in [&mut short, &mut game] {
        passes(branch);
        settle(branch);
    }
    assert!(tokens(&short, Card(5307)).is_empty());
    let elementals = tokens(&game, Card(5307));
    assert_eq!(elementals.len(), 1);
    assert_eq!(stats(&game, elementals[0]), (2, 2));
    assert_eq!(game.object_view(elementals[0]).colors, 31);
    assert!(game.has_subtype(elementals[0], "Elemental"));
    game.state.players[player.index()].mana = [1, 1, 1, 1, 1, 3];
    announce(&mut game, minstrel, 0);
    pay(&mut game);
    let mut fewer = game.clone();
    fewer.move_object(old_town, Zone::Exile);
    settle(&mut fewer);
    settle(&mut game);
    assert_eq!(stats(&fewer, ally), (6, 6));
    assert_eq!(stats(&game, ally), (7, 7));
    assert_eq!(stats(&game, minstrel), (1, 3));
}

#[test]
fn treno_vector_and_swamp_have_only_their_printed_mana_choices() {
    let player = Player::P0;
    for (name, colors) in [
        ("Treno, Dark City", [1, 2]),
        ("Vector, Imperial Capital", [2, 3]),
    ] {
        let mut game = scenario(player);
        let land = put(&mut game, player, name, Zone::Hand);
        act(&mut game, Action::PlayLand { card: land });
        assert!(game.object_view(land).tapped);
        let produce = |color| {
            let mut production = [0; 6];
            production[color] = 1;
            Action::ActivateMana {
                card: land,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production,
            }
        };
        reject(&mut game, produce(colors[0]));
        game.untap_active_player();
        reject(&mut game, produce(0));
        for color in colors {
            let mut branch = game.clone();
            act(&mut branch, produce(color));
            assert_eq!(branch.state.players[player.index()].mana[color], 9);
        }
    }
    let mut game = scenario(player);
    let swamp = put(&mut game, player, "Swamp", Zone::Hand);
    act(&mut game, Action::PlayLand { card: swamp });
    assert!(!game.object_view(swamp).tapped);
    act(&mut game, Action::TapMana { card: swamp });
    assert_eq!(game.state.players[player.index()].mana, [8, 8, 9, 8, 8, 8]);
}

#[test]
fn tidus_counts_own_artifact_entries_and_ultimecia_and_thiefs_knife_draw_for_combat() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let tidus = put(
        &mut game,
        player,
        "Tidus, Blitzball Star",
        Zone::Battlefield,
    );
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let enemy_artifact = put(&mut game, opponent, "Swiftfoot Boots", Zone::Battlefield);
    let ultimecia = put(&mut game, player, "Ultimecia, Temporal Threat", Zone::Hand);
    cast(&mut game, ultimecia, None);
    settle(&mut game);
    assert!(game.object_view(enemy).tapped);
    assert!(!game.object_view(enemy_artifact).tapped);
    assert!(!game.object_view(ally).tapped);
    assert_eq!(game.counter_count(tidus, CounterKind::PlusOne), 0);
    let knife = put(&mut game, player, "Thief's Knife", Zone::Hand);
    cast(&mut game, knife, None);
    passes(&mut game);
    if matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    settle(&mut game);
    let hero = tokens(&game, Card(5328))[0];
    assert_eq!(
        game.object_view(knife).attached_to,
        Some(game.object_ref(hero))
    );
    assert_eq!(stats(&game, hero), (2, 2));
    assert!(game.has_subtype(hero, "Rogue"));
    assert_eq!(game.counter_count(tidus, CounterKind::PlusOne), 1);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 4];
    announce(&mut game, knife, 0);
    targets(&mut game, Target::Card(tidus));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert!(game.has_subtype(tidus, "Rogue"));
    assert!(!game.has_subtype(hero, "Rogue"));
    let late_enemy = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    attack(&mut game, &[tidus, ally]);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(ally),
        },
    );
    targets(&mut game, Target::Card(late_enemy));
    settle(&mut game);
    assert!(game.object_view(late_enemy).tapped);
    passes(&mut game);
    act(&mut game, Action::FinishBlockers);
    passes(&mut game);
    while matches!(game.state.pending, Pending::OrderTriggers { .. }) {
        choose(&mut game, |action| {
            matches!(action, Action::PutTrigger { .. })
        });
    }
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 14);
    assert_eq!(game.hand_count(player), 3);
}

#[test]
fn thunder_magic_and_tifas_limit_break_pay_every_tier_and_resolve_its_printed_value() {
    let player = Player::P0;
    for (mode, generic, red, damage) in [(0, 0, 1, 2), (1, 3, 1, 4), (2, 5, 2, 8)] {
        let mut game = scenario(player);
        let target = put(
            &mut game,
            player.other(),
            "Gigantosaurus",
            Zone::Battlefield,
        );
        let spell = put(&mut game, player, "Thunder Magic", Zone::Hand);
        modal(&mut game, spell, mode, Target::Card(target));
        let request = game.observe(player).payment_request.unwrap();
        assert_eq!((request.generic, request.required[3]), (generic, red));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.object_view(target).damage, damage);
    }
    for (mode, generic, green, expected) in
        [(0, 0, 1, (4, 4)), (1, 2, 1, (-2, 4)), (2, 6, 2, (6, 6))]
    {
        let mut game = scenario(player);
        let target = put(&mut game, player, "Bear Cub", Zone::Battlefield);
        if mode == 1 {
            game.state.objects[target as usize].growth.power = -3;
        }
        let spell = put(&mut game, player, "Tifa's Limit Break", Zone::Hand);
        modal(&mut game, spell, mode, Target::Card(target));
        let request = game.observe(player).payment_request.unwrap();
        assert_eq!((request.generic, request.required[4]), (generic, green));
        pay(&mut game);
        settle(&mut game);
        assert_eq!(stats(&game, target), expected);
    }
}

#[test]
fn ultima_weapon_triggers_for_its_controller_even_when_the_attacker_is_stolen() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let weapon = put(&mut game, player, "Ultima Weapon", Zone::Battlefield);
    let ally = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let victim = put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 7];
    announce(&mut game, weapon, 0);
    targets(&mut game, Target::Card(attacker));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(stats(&game, attacker), (9, 9));
    game.record_control(game.object_ref(attacker), opponent, None);
    game.state.objects[attacker as usize].summoning_sick = false;
    game.state.active = opponent;
    game.give_priority();
    attack(&mut game, &[attacker]);
    let target = Action::ChooseTarget {
        target: Target::Card(victim),
    };
    reach(&mut game, &target);
    assert_eq!(game.actor(), Some(player));
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(ally),
        },
    );
    targets(&mut game, Target::Card(victim));
    game.move_object(weapon, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(stats(&game, attacker), (2, 2));
}

#[test]
fn ultros_has_distinct_four_and_eight_mana_triggers() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ultros = put(
        &mut game,
        player,
        "Ultros, Obnoxious Octopus",
        Zone::Battlefield,
    );
    let enemy = put(&mut game, player.other(), "Bear Cub", Zone::Battlefield);
    let ring = put(&mut game, player, "Ring of the Lucii", Zone::Hand);
    cast(&mut game, ring, None);
    let target = Action::ChooseTarget {
        target: Target::Card(enemy),
    };
    reach(&mut game, &target);
    targets(&mut game, Target::Card(enemy));
    settle(&mut game);
    assert_eq!(game.counter_count(enemy, CounterKind::Stun), 1);
    assert_eq!(game.counter_count(ultros, CounterKind::PlusOne), 0);
    let limit = put(&mut game, player, "Tifa's Limit Break", Zone::Hand);
    game.state.players[player.index()].mana = [8; 6];
    modal(&mut game, limit, 2, Target::Card(ultros));
    pay(&mut game);
    assert_eq!(game.state.ordering_triggers.len(), 2);
    reach(&mut game, &target);
    targets(&mut game, Target::Card(enemy));
    let mut departed = game.clone();
    departed.move_object(enemy, Zone::Exile);
    settle(&mut departed);
    assert_eq!(departed.counter_count(ultros, CounterKind::PlusOne), 8);
    settle(&mut game);
    assert!(game.object_view(enemy).tapped);
    assert_eq!(game.counter_count(enemy, CounterKind::Stun), 2);
    assert_eq!(game.counter_count(ultros, CounterKind::PlusOne), 8);
}

#[test]
fn valkyrie_affinity_counts_an_artifact_creature_once_and_surveys_privately() {
    let player = Player::P0;
    let mut game = scenario(player);
    for _ in 0..4 {
        put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    }
    put(&mut game, player, "Scorpion Sentinel", Zone::Battlefield);
    put(
        &mut game,
        player.other(),
        "Swiftfoot Boots",
        Zone::Battlefield,
    );
    let kept = put(&mut game, player, "Island", Zone::Library);
    let top = put(&mut game, player, "Shock", Zone::Library);
    let airship = put(&mut game, player, "Valkyrie Aerial Unit", Zone::Hand);
    game.state.players[player.index()].mana = [0, 2, 0, 0, 0, 0];
    act(&mut game, Action::AnnounceCast { card: airship });
    let request = game.observe(player).payment_request.unwrap();
    assert_eq!((request.generic, request.required), (0, [0, 2, 0, 0, 0, 0]));
    pay(&mut game);
    let mill = Action::ArrangeLibrary {
        card: top,
        destination: LibraryDestination::Graveyard,
    };
    reach(&mut game, &mill);
    assert!(game.observe(player.other()).library_choice.is_none());
    act(&mut game, mill);
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: kept,
            destination: LibraryDestination::Top,
        },
    );
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
    assert_eq!(
        game.state.players[player.index()].library.last(),
        Some(&kept)
    );
    assert_eq!(stats(&game, airship), (5, 4));
    assert!(game.keywords(airship).contains(Keywords::FLYING));
}
