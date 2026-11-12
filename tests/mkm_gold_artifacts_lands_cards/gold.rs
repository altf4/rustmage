use super::support::*;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};

const CLUE: Card = Card::from_catalogue_id(5296);

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target(game: &mut Game, target: Target) {
    act(game, Action::ChooseTarget { target });
}

fn choose_cast_cost(game: &mut Game, clause: CastCostClause) {
    let choice = game
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == clause)
        .unwrap();
    act(
        game,
        Action::ChooseCastCost {
            clause: choice.try_into().unwrap(),
        },
    );
}

fn exact_cost_object(game: &Game, card: CardId) -> Action {
    Action::ChooseAdditionalCost {
        object: game.object_ref(card),
    }
}

fn tokens(game: &Game, player: Player, recipe: Card) -> usize {
    game.controlled_token_ids(player)
        .filter(|&id| game.object_view(id).card == recipe)
        .count()
}

fn combat_with_attackers(game: &mut Game, attackers: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn until(game: &mut Game, condition: impl Fn(&Game) -> bool) {
    for _ in 0..20 {
        if condition(game) {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::FinishBlockers
                    | Action::ChooseEntryTimestamp { .. }
                    | Action::PutTrigger { .. }
            )
        });
    }
    panic!(
        "scenario did not reach its expected decision: {:?}",
        game.state.pending
    );
}

#[test]
fn niv_counts_distinct_pairs_once_for_damage_draw_and_life() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let niv = put(
        &mut game,
        player,
        "Niv-Mizzet, Guildpact",
        Zone::Battlefield,
    );
    assert!(game.keywords(niv).contains(Keywords::FLYING));
    game.create_tokens(player, Card::from_catalogue_id(5329), 1, false);
    assert!(!game.advance_entries());
    for name in [
        "Warleader's Call",
        "Warleader's Call",
        "Trostani, Three Whispers",
        "Riku of Many Paths",
    ] {
        put(&mut game, player, name, Zone::Battlefield);
    }

    for (spell_name, legal) in [("Lightning Helix", false), ("Shock", true)] {
        let mut choice = game.clone();
        choice.state.priority = opponent;
        let spell = put(&mut choice, opponent, spell_name, Zone::Hand);
        act(&mut choice, Action::AnnounceCast { card: spell });
        let aimed = Action::ChooseTarget {
            target: Target::Card(niv),
        };
        assert_eq!(choice.legal_actions().contains(&aimed), legal);
    }

    combat_with_attackers(&mut game, &[niv]);
    until(&mut game, |g| {
        matches!(g.state.pending, Pending::Targets { .. })
    });
    assert_eq!(game.state.players[opponent.index()].life, 12);
    target(&mut game, Target::Player(opponent));
    target(&mut game, Target::Player(player));
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 9);
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 3);
}

#[test]
fn officious_interrogation_locks_surcharges_and_investigates_the_live_count() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    put(&mut game, opponent, "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Officious Interrogation", Zone::Hand);

    let mut one = game.clone();
    one.state.players[player.index()].mana = [1, 1, 0, 0, 0, 0];
    act(&mut one, Action::AnnounceCast { card: spell });
    target(&mut one, Target::Player(opponent));
    reject(
        &mut one,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    act(&mut one, Action::FinishTargets);
    pay(&mut one);
    settle(&mut one);
    assert_eq!(tokens(&one, player, CLUE), 2);

    game.state.players[player.index()].mana = [2, 2, 0, 0, 0, 0];
    announce(
        &mut game,
        spell,
        &[Target::Player(opponent), Target::Player(player)],
    );
    let paid = game.state.stack.last().unwrap().cast.unwrap().payment;
    assert_eq!(paid, [2, 2, 0, 0, 0, 0]);
    game.move_object(attacker, Zone::Exile);
    settle(&mut game);
    assert_eq!(tokens(&game, player, CLUE), 2);
}

#[test]
fn private_eye_buffs_only_other_controlled_detectives_and_targets_on_draw_two() {
    let player = Player::P1;
    let mut game = scenario(player);
    let eye = put(&mut game, player, "Private Eye", Zone::Battlefield);
    let own = put(&mut game, player, "Novice Inspector", Zone::Battlefield);
    let opposing = put(
        &mut game,
        player.other(),
        "Novice Inspector",
        Zone::Battlefield,
    );
    let other = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    assert_eq!(game.stats(eye).power, 3);
    assert_eq!(game.stats(own).power, 2);
    assert_eq!(game.stats(opposing).power, 1);
    assert_eq!(game.stats(other).power, 2);
    let draw = put(&mut game, player, "Quick Study", Zone::Hand);
    cast(&mut game, draw, None);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(other),
        },
    );
    let mut branch = game.clone();
    target(&mut branch, Target::Card(opposing));
    settle(&mut branch);
    assert!(
        branch
            .keywords(opposing)
            .contains(Keywords::CANT_BE_BLOCKED)
    );
    target(&mut game, Target::Card(own));
    settle(&mut game);
    assert!(game.keywords(own).contains(Keywords::CANT_BE_BLOCKED));
    let draw = put(&mut game, player, "Quick Study", Zone::Hand);
    cast(&mut game, draw, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 4);
}

#[test]
fn rakdos_targets_an_opponent_who_can_pay_two_actual_eligible_permanents() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let rakdos = put(
        &mut game,
        player,
        "Rakdos, Patron of Chaos",
        Zone::Battlefield,
    );
    assert!(
        game.keywords(rakdos)
            .contains(Keywords::FLYING.union(Keywords::TRAMPLE))
    );
    let first = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, opponent, "Short Bow", Zone::Battlefield);
    let land = put(&mut game, opponent, "Forest", Zone::Battlefield);
    game.create_tokens(opponent, CLUE, 1, false);
    assert!(!game.advance_entries());
    let token = game.controlled_token_ids(opponent).next().unwrap();
    game.begin_step(Step::End);
    game.prepare_triggers();
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    target(&mut game, Target::Player(opponent));
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    for id in [land, token] {
        let action = exact_cost_object(&game, id);
        reject(&mut game, action);
    }
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineResolutionMana);
    assert_eq!(
        decline.state.players[player.index()].cards_drawn_this_turn,
        2
    );
    for id in [first, second] {
        let action = exact_cost_object(&game, id);
        act(&mut game, action);
    }
    act(&mut game, Action::PayResolutionMana { payment: [0; 6] });
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    for id in [first, second] {
        assert_eq!(game.object_view(id).zone, Zone::Graveyard);
    }
}

#[test]
fn relive_returns_each_legal_role_as_a_five_five_elemental() {
    let player = Player::P0;
    let mut game = scenario(player);
    let equipment = put(&mut game, player, "Swiftfoot Boots", Zone::Graveyard);
    let land = put(&mut game, player, "Forest", Zone::Graveyard);
    let enchantment = put(&mut game, player, "Impact Tremors", Zone::Graveyard);
    let aura = put(&mut game, player, "Pacifism", Zone::Graveyard);
    let relive = put(&mut game, player, "Relive the Past", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: relive });
    target(&mut game, Target::Card(equipment));
    target(&mut game, Target::Card(land));
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(aura),
        },
    );
    target(&mut game, Target::Card(enchantment));
    pay(&mut game);
    until(&mut game, |g| {
        g.state.pending == Pending::Priority && g.state.stack.is_empty()
    });
    for (id, kinds) in [(equipment, 17), (land, 3), (enchantment, 33)] {
        let object = game.object_view(id);
        assert_eq!(object.zone, Zone::Battlefield);
        assert_eq!(object.kinds, kinds);
        assert!(object.has_subtype("Elemental"));
        assert_eq!((object.power, object.toughness), (5, 5));
    }
    assert_eq!(game.state.players[player.other().index()].life, 17);
}

#[test]
fn repulsive_mutation_counts_power_after_the_new_counters() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let recipient = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    game.state.priority = opponent;
    game.state.players[opponent.index()].mana = [0, 0, 0, 1, 0, 4];
    cast(&mut game, shock, Some(Target::Player(player)));
    act(&mut game, Action::Pass);
    let mutation = put(&mut game, player, "Repulsive Mutation", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: mutation });
    act(&mut game, Action::ChooseX { value: 2 });
    target(&mut game, Target::Card(recipient));
    target(&mut game, Target::Spell(shock));
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 2);
    reject(
        &mut game,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 3],
        },
    );
    let mut paid = game.clone();
    act(
        &mut paid,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 4],
        },
    );
    settle(&mut paid);
    assert_eq!(paid.state.players[player.index()].life, 18);
    act(&mut game, Action::DeclineResolutionMana);
    assert!(game.state.stack.is_empty());
    assert_eq!(game.state.players[player.index()].life, 20);
}

fn activate(game: &mut Game, source: CardId, index: u16) {
    choose(game, |action| {
        matches!(
            action,
            Action::Activate { card, ability, .. } | Action::AnnounceActivate { card, ability, .. }
            if *card == source && *ability == index
        )
    });
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

#[test]
fn rune_brand_juggler_suspects_then_requires_that_designation_as_a_cost() {
    let player = Player::P0;
    let mut game = scenario(player);
    let suspected = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let innocent = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let juggler = put(&mut game, player, "Rune-Brand Juggler", Zone::Hand);
    cast(&mut game, juggler, None);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    target(&mut game, Target::Card(suspected));
    settle(&mut game);
    assert!(game.is_suspected(suspected));

    act(
        &mut game,
        Action::AnnounceActivate {
            card: juggler,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    target(&mut game, Target::Card(enemy));
    let illegal = exact_cost_object(&game, innocent);
    reject(&mut game, illegal);
    let cost = exact_cost_object(&game, suspected);
    act(&mut game, cost);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(suspected).zone, Zone::Graveyard);
    assert_eq!(game.object_view(enemy).zone, Zone::Graveyard);
    assert!(game.on_battlefield(innocent));
}

#[test]
fn soul_search_reveals_the_hand_and_rewards_only_a_small_nonland() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let land = put(&mut game, opponent, "Forest", Zone::Hand);
    let small = put(&mut game, opponent, "Shock", Zone::Hand);
    let large = put(&mut game, opponent, "Grizzly Bears", Zone::Hand);
    let search = put(&mut game, player, "Soul Search", Zone::Hand);
    cast(&mut game, search, Some(Target::Player(opponent)));
    pass_pair(&mut game);
    let selection = |id| Action::ChooseHandCard {
        object: Some(game.object_ref(id)),
    };
    let land = selection(land);
    let small_action = selection(small);
    let large_action = selection(large);
    assert_eq!(game.observe(player).known_hand.len(), 3);
    reject(&mut game, land);
    reject(&mut game, Action::ChooseHandCard { object: None });
    let spirit = Card::from_catalogue_id(5477);
    let mut expensive = game.clone();
    act(&mut expensive, large_action);
    assert_eq!(expensive.object_view(large).zone, Zone::Exile);
    assert_eq!(tokens(&expensive, player, spirit), 0);
    act(&mut game, small_action);
    assert_eq!(game.object_view(small).zone, Zone::Exile);
    let id = game.controlled_token_ids(player).next().unwrap();
    let token = game.object_view(id);
    assert_eq!(token.card, spirit);
    assert_eq!(token.card.colors(), 1 | 4);
    assert_eq!((token.power, token.toughness), (1, 1));
    assert!(token.keywords.contains(Keywords::FLYING));
}

#[test]
fn teysa_checks_life_lost_at_resolution_and_triggers_for_the_first_clue_death() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let teysa = put(
        &mut game,
        player,
        "Teysa, Opulent Oligarch",
        Zone::Battlefield,
    );
    assert!(game.keywords(teysa).contains(Keywords::DEATHTOUCH));
    game.begin_step(Step::End);
    game.prepare_triggers();
    let mut no_loss = game.clone();
    settle(&mut no_loss);
    assert_eq!(tokens(&no_loss, player, CLUE), 0);

    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(opponent)));
    settle(&mut game);
    assert_eq!(tokens(&game, player, CLUE), 1);
    game.create_tokens(player, CLUE, 1, false);
    assert!(!game.advance_entries());
    let clues: Vec<_> = game.controlled_token_ids(player).collect();
    for id in clues {
        activate(&mut game, id, 0);
        settle(&mut game);
    }
    assert_eq!(tokens(&game, player, Card::from_catalogue_id(5477)), 1);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

#[test]
fn tolsimir_creates_the_named_wolf_and_pairs_its_attack_with_a_blocker() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let tolsimir = put(&mut game, player, "Tolsimir, Midnight's Light", Zone::Hand);
    cast(&mut game, tolsimir, None);
    settle(&mut game);
    assert!(game.keywords(tolsimir).contains(Keywords::LIFELINK));
    let wolf = game.controlled_token_ids(player).next().unwrap();
    let object = game.object_view(wolf);
    assert_eq!(object.card.name(), "Voja Fenstalker");
    assert!(object.card.is_legendary());
    assert_eq!(object.card.colors(), 1 | 16);
    assert!(object.has_subtype("Wolf"));
    assert!(object.keywords.contains(Keywords::TRAMPLE));
    assert_eq!((object.power, object.toughness), (5, 5));

    for id in [wolf, tolsimir] {
        game.state.objects[id as usize].summoning_sick = false;
    }
    let blocker = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let mut alone = game.clone();
    combat_with_attackers(&mut alone, &[wolf]);
    assert!(alone.state.stack.is_empty());

    combat_with_attackers(&mut game, &[wolf, tolsimir]);
    target(&mut game, Target::Card(blocker));
    settle(&mut game);
    until(&mut game, |g| g.state.pending == Pending::Blockers);
    reject(&mut game, Action::FinishBlockers);
    act(
        &mut game,
        Action::Block {
            blocker,
            attacker: wolf,
        },
    );
    act(&mut game, Action::FinishBlockers);
}

#[test]
fn treacherous_greed_pays_with_the_creature_that_actually_dealt_damage() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let dealer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let idle = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let greed = put(&mut game, player, "Treacherous Greed", Zone::Hand);
    reject(&mut game, Action::AnnounceCast { card: greed });
    combat_with_attackers(&mut game, &[dealer]);
    until(&mut game, |g| g.state.step == Step::CombatDamage);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    game.state.players[player.index()].mana = [8; 6];
    act(&mut game, Action::AnnounceCast { card: greed });
    let forbidden = exact_cost_object(&game, idle);
    reject(&mut game, forbidden);
    let sacrifice = exact_cost_object(&game, dealer);
    act(&mut game, sacrifice);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(dealer).zone, Zone::Graveyard);
    assert!(game.on_battlefield(idle));
    assert_eq!(game.state.players[opponent.index()].life, 15);
    assert_eq!(game.state.players[player.index()].life, 23);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 3);
}

#[test]
fn urgent_necropsy_collects_the_sum_of_the_selected_roles_and_destroys_once() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let creature = put(&mut game, opponent, "Gleaming Geardrake", Zone::Battlefield);
    let four = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let two = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let spell = put(&mut game, player, "Urgent Necropsy", Zone::Hand);
    let old = game.object_ref(creature);
    act(&mut game, Action::AnnounceCast { card: spell });
    for _ in 0..2 {
        target(&mut game, Target::Card(creature));
    }
    for _ in 0..2 {
        if !game.legal_actions().contains(&Action::FinishTargets) {
            break;
        }
        act(&mut game, Action::FinishTargets);
    }
    act(&mut game, Action::BeginOrderedCosts);
    choose_cast_cost(&mut game, CastCostClause::Additional);
    let evidence = |g: &Game, id| Action::ChooseEvidence {
        object: g.object_ref(id),
    };
    let small = evidence(&game, two);
    let mut short = game.clone();
    act(&mut short, small);
    reject(&mut short, Action::FinishEvidence);
    let full = evidence(&game, four);
    act(&mut game, full);
    act(&mut game, Action::FinishEvidence);
    choose_cast_cost(&mut game, CastCostClause::Mana);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(four).zone, Zone::Exile);
    assert_eq!(game.object_view(two).zone, Zone::Graveyard);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    assert_eq!(game.object_ref(creature).generation, old.generation + 1);
}

#[test]
fn wispdrinker_uses_entry_power_and_freezes_the_activation_recipients() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let vampire = put(&mut game, player, "Wispdrinker Vampire", Zone::Battlefield);
    assert!(game.keywords(vampire).contains(Keywords::FLYING));
    let small = put(&mut game, player, "Grizzly Bears", Zone::Hand);
    cast(&mut game, small, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 21);
    assert_eq!(game.state.players[opponent.index()].life, 19);

    let large = put(&mut game, player, "Hill Giant", Zone::Hand);
    cast(&mut game, large, None);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 19);

    game.state.players[player.index()].mana = [8; 6];
    activate(&mut game, vampire, 0);
    settle(&mut game);
    let granted = Keywords::DEATHTOUCH.union(Keywords::LIFELINK);
    assert!(game.keywords(small).contains(granted));
    assert!(game.keywords(vampire).contains(granted));
    assert!(!game.keywords(large).intersects(granted));
    let growth = put(&mut game, player, "Giant Growth", Zone::Hand);
    cast(&mut game, growth, Some(Target::Card(small)));
    settle(&mut game);
    assert!(game.keywords(small).contains(granted));
}

#[test]
fn melek_counts_actual_instant_and_sorcery_cards_and_reduces_only_the_first_cast() {
    let player = Player::P0;
    let mut game = scenario(player);
    put(&mut game, player, "Shock", Zone::Graveyard);
    put(&mut game, player, "Flotsam // Jetsam", Zone::Graveyard);
    put(
        &mut game,
        player,
        "Questing Druid // Seek the Beast",
        Zone::Graveyard,
    );
    let melek = put(
        &mut game,
        player,
        "Melek, Reforged Researcher",
        Zone::Battlefield,
    );
    assert_eq!(
        (game.stats(melek).power, game.stats(melek).toughness),
        (4, 4)
    );
    let first = put(&mut game, player, "Quick Study", Zone::Hand);
    let second = put(&mut game, player, "Quick Study", Zone::Hand);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    cast(&mut game, first, None);
    let receipt = game.state.stack.last().unwrap().cast.unwrap();
    assert_eq!(receipt.payment, [0, 1, 0, 0, 0, 0]);
    settle(&mut game);
    assert_eq!(game.stats(melek).power, 6);
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    reject(&mut game, Action::AnnounceCast { card: second });

    game.state.step = Step::End;
    pass_pair(&mut game);
    assert_eq!(game.state.active, player.other());
    if game.actor() == Some(player.other()) {
        act(&mut game, Action::Pass);
    }
    game.state.players[player.index()].mana = [0, 1, 0, 0, 0, 0];
    cast(&mut game, second, None);
    settle(&mut game);
    assert_eq!(game.stats(melek).power, 8);
}

#[test]
fn tomik_uses_planeswalker_affinity_and_counts_both_defended_destinations() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let walker = put(
        &mut game,
        player,
        "Kaya, Spirits' Justice",
        Zone::Battlefield,
    );
    game.set_counters(walker, Counters::of(CounterKind::Loyalty, 3));
    let tomik = put(&mut game, player, "Tomik, Wielder of Law", Zone::Hand);
    game.state.players[player.index()].mana = [1, 0, 1, 0, 0, 0];
    cast(&mut game, tomik, None);
    settle(&mut game);
    assert!(
        game.keywords(tomik)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);

    let first = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    game.state.active = opponent;
    game.state.priority = opponent;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    let mut one = game.clone();
    act(&mut one, Action::Attack { card: first });
    act(&mut one, Action::FinishAttackers);
    assert!(one.state.stack.is_empty());

    act(&mut game, Action::Attack { card: first });
    let walker = game.object_ref(walker);
    act(
        &mut game,
        Action::AttackPlaneswalker {
            card: second,
            defender: walker,
        },
    );
    act(&mut game, Action::FinishAttackers);
    settle(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 17);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 1);
}

#[test]
fn voja_counts_the_live_elves_and_wolves_and_has_ward_three() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let voja = put(
        &mut game,
        player,
        "Voja, Jaws of the Conclave",
        Zone::Battlefield,
    );
    assert!(
        game.keywords(voja)
            .contains(Keywords::VIGILANCE.union(Keywords::TRAMPLE))
    );
    let first = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let second = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    game.create_tokens(player, Card::from_catalogue_id(5686), 1, false);
    assert!(!game.advance_entries());
    let wolf = game.controlled_token_ids(player).next().unwrap();

    let mut ward = game.clone();
    ward.state.priority = opponent;
    let shock = put(&mut ward, opponent, "Shock", Zone::Hand);
    cast(&mut ward, shock, Some(Target::Card(voja)));
    pass_pair(&mut ward);
    reject(
        &mut ward,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 2],
        },
    );
    act(&mut ward, Action::DeclineResolutionMana);
    assert_eq!(ward.object_view(shock).zone, Zone::Graveyard);
    assert_eq!(ward.object_view(voja).damage, 0);

    combat_with_attackers(&mut game, &[voja]);
    settle(&mut game);
    assert!(!game.object_view(voja).tapped);
    for id in [voja, first, second, wolf] {
        assert_eq!(game.counter_count(id, CounterKind::PlusOne), 2);
    }
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

#[test]
fn no_more_lies_exiles_the_exact_spell_unless_its_controller_pays_three() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    game.state.priority = opponent;
    game.state.players[opponent.index()].mana = [0, 0, 0, 1, 0, 3];
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(player)));
    act(&mut game, Action::Pass);
    let counter = put(&mut game, player, "No More Lies", Zone::Hand);
    cast(&mut game, counter, Some(Target::Spell(shock)));
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));

    let mut paid = game.clone();
    act(
        &mut paid,
        Action::PayResolutionMana {
            payment: [0, 0, 0, 0, 0, 3],
        },
    );
    settle(&mut paid);
    assert_eq!(paid.object_view(shock).zone, Zone::Graveyard);
    assert_eq!(paid.state.players[player.index()].life, 18);

    act(&mut game, Action::DeclineResolutionMana);
    assert!(game.state.stack.is_empty());
    assert_eq!(game.object_view(shock).zone, Zone::Exile);
    assert_eq!(game.state.players[player.index()].life, 20);
}

#[test]
fn trostani_uses_both_hybrid_payments_and_grants_all_three_keywords_until_cleanup() {
    let player = Player::P0;
    let mut game = scenario(player);
    let trostani = put(&mut game, player, "Trostani, Three Whispers", Zone::Hand);
    for (cost, mana) in [[1, 0, 0, 0, 2, 0], [2, 0, 0, 0, 1, 0]]
        .into_iter()
        .enumerate()
    {
        let mut branch = game.clone();
        branch.state.players[player.index()].mana = mana;
        act(&mut branch, Action::AnnounceCast { card: trostani });
        if matches!(branch.state.pending, Pending::SpellCost { .. }) {
            act(
                &mut branch,
                Action::ChooseSpellCost {
                    cost: cost.try_into().unwrap(),
                },
            );
        }
        pay(&mut branch);
        settle(&mut branch);
        assert_eq!(stats(&branch, trostani), (4, 4));
    }
    cast(&mut game, trostani, None);
    settle(&mut game);
    let recipient = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    for (ability, keyword) in [
        (0, Keywords::DEATHTOUCH),
        (1, Keywords::VIGILANCE),
        (2, Keywords::DOUBLE_STRIKE),
    ] {
        act(
            &mut game,
            Action::AnnounceActivate {
                card: trostani,
                ability,
                counter_payment: Counters::default(),
            },
        );
        if matches!(game.state.pending, Pending::ActivationCost { .. }) {
            let mut green = game.clone();
            for (branch, cost) in [(&mut green, 0), (&mut game, 1)] {
                act(branch, Action::ChooseActivationCost { cost });
                target(branch, Target::Card(recipient));
                pay(branch);
                settle(branch);
                assert!(branch.keywords(recipient).contains(keyword));
            }
        } else {
            target(&mut game, Target::Card(recipient));
            pay(&mut game);
            settle(&mut game);
        }
        assert!(game.keywords(recipient).contains(keyword));
        assert!(!game.keywords(trostani).contains(keyword));
    }
    game.finish_cleanup();
    assert!(!game.keywords(recipient).contains(Keywords::DEATHTOUCH));
    assert!(!game.keywords(recipient).contains(Keywords::VIGILANCE));
    assert!(!game.keywords(recipient).contains(Keywords::DOUBLE_STRIKE));
}

#[test]
fn warleaders_call_pumps_only_its_creatures_and_damages_for_each_controlled_entry() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let call = put(&mut game, player, "Warleader's Call", Zone::Hand);
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let opposing = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    cast(&mut game, call, None);
    settle(&mut game);
    assert_eq!(stats(&game, own), (3, 3));
    assert_eq!(stats(&game, opposing), (2, 2));
    assert_eq!(game.state.players[opponent.index()].life, 20);
    for (name, life) in [("Grizzly Bears", 19), ("Bear Cub", 18)] {
        let creature = put(&mut game, player, name, Zone::Hand);
        cast(&mut game, creature, None);
        settle(&mut game);
        assert_eq!(stats(&game, creature), (3, 3));
        assert_eq!(game.state.players[opponent.index()].life, life);
    }
    assert_eq!(game.state.players[player.index()].life, 20);
}

#[test]
fn worldsouls_rage_deals_x_then_puts_up_to_x_lands_from_hand_and_graveyard_tapped() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let rage = put(&mut game, player, "Worldsoul's Rage", Zone::Hand);
    let hand_land = put(&mut game, player, "Mountain", Zone::Hand);
    let grave_land = put(&mut game, player, "Forest", Zone::Graveyard);
    let third_land = put(&mut game, player, "Plains", Zone::Hand);
    let opposing_land = put(&mut game, opponent, "Island", Zone::Graveyard);
    act(&mut game, Action::AnnounceCast { card: rage });
    act(&mut game, Action::ChooseX { value: 2 });
    target(&mut game, Target::Player(opponent));
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    let foreign = game.object_ref(opposing_land);
    reject(&mut game, Action::SelectLand { object: foreign });
    let mut none = game.clone();
    act(&mut none, Action::FinishLandSelection);
    assert_eq!(none.object_view(grave_land).zone, Zone::Graveyard);

    for land in [hand_land, grave_land] {
        let object = game.object_ref(land);
        act(&mut game, Action::SelectLand { object });
    }
    if game.legal_actions().contains(&Action::FinishLandSelection) {
        act(&mut game, Action::FinishLandSelection);
    }
    settle(&mut game);
    for land in [hand_land, grave_land] {
        assert!(game.on_battlefield(land));
        assert!(game.object_view(land).tapped);
    }
    assert_eq!(game.object_view(third_land).zone, Zone::Hand);
}

#[test]
fn vannifar_cloaks_a_private_hand_card_or_counts_each_live_colorless_creature() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(
        &mut game,
        player,
        "Vannifar, Evolved Enigma",
        Zone::Battlefield,
    );
    let to_cloak = put(&mut game, player, "Soul-Guide Lantern", Zone::Hand);
    put(&mut game, player, "Forest", Zone::Hand);
    let hostile_hand = put(&mut game, opponent, "Shock", Zone::Hand);
    let colored = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let noncreature = put(&mut game, player, "Soul-Guide Lantern", Zone::Battlefield);
    let thopter = Card::from_catalogue_id(5266);
    game.create_tokens(player, thopter, 1, false);
    assert!(!game.advance_entries());
    let friendly_token = game.controlled_token_ids(player).next().unwrap();
    game.create_tokens(opponent, thopter, 1, false);
    assert!(!game.advance_entries());
    let hostile_token = game.controlled_token_ids(opponent).next().unwrap();
    let hidden = put(&mut game, player, "Bear Cub", Zone::Library);
    game.cloak_library_top(player, player);
    assert!(!game.advance_entries());

    let mut other_turn = game.clone();
    other_turn.state.active = opponent;
    other_turn.state.priority = opponent;
    pass_pair(&mut other_turn);
    assert!(other_turn.state.stack.is_empty());
    assert_eq!(other_turn.state.pending, Pending::Priority);

    pass_pair(&mut game);
    let mut cloak = game.clone();
    act(&mut cloak, Action::ChooseMode { mode: 0 });
    pass_pair(&mut cloak);
    assert!(cloak.observe(player).effect_choice.is_some());
    assert!(cloak.observe(opponent).effect_choice.is_none());
    let foreign = cloak.object_ref(hostile_hand);
    reject(&mut cloak, Action::ChooseEffectObject { object: foreign });
    let object = cloak.object_ref(to_cloak);
    act(&mut cloak, Action::ChooseEffectObject { object });
    let face = cloak
        .observe(player)
        .face_down_permanents
        .into_iter()
        .find(|row| row.underlying == Card::from_name("Soul-Guide Lantern"))
        .unwrap();
    assert_eq!(face.cause, FaceDownCause::Cloak);
    assert_ne!(face.object.card, to_cloak);
    assert!(cloak.on_battlefield(face.object.card));
    assert_eq!(stats(&cloak, face.object.card), (2, 2));
    assert!(
        cloak
            .observe(opponent)
            .face_down_permanents
            .iter()
            .all(|row| row.underlying.is_none())
    );

    act(&mut game, Action::ChooseMode { mode: 1 });
    settle(&mut game);
    for card in [friendly_token, hidden] {
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 1);
    }
    for card in [source, hostile_token, colored, noncreature] {
        assert_eq!(game.counter_count(card, CounterKind::PlusOne), 0);
    }
}
