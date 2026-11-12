use super::{activation, additional, begin_attack, enter_step, life, pass_pair, target, tokens};
use crate::support::*;
use rustmage_core::{cards::*, engine::*};

const THOPTER: Card = Card::from_catalogue_id(5266);
const GOBLIN: Card = Card::from_catalogue_id(5324);

fn evidence(game: &mut Game, cards: &[CardId]) {
    for &card in cards {
        let object = game.object_ref(card);
        act(game, Action::ChooseEvidence { object });
    }
    act(game, Action::FinishEvidence);
}

fn pick(game: &mut Game, card: CardId) {
    let object = game.object_ref(card);
    act(game, Action::ChooseEffectObject { object });
}

#[test]
fn illicit_masquerade_marks_only_your_creatures_and_exiles_the_exact_death() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(opponent);
    let enchantment = put(&mut game, player, "Illicit Masquerade", Zone::Hand);
    let victim = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let companion = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let other = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let returnable = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let removal = put(&mut game, opponent, "Murder", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, enchantment, None);
    settle(&mut game);
    assert_eq!(game.counter_count(victim, CounterKind::Impostor), 1);
    assert_eq!(game.counter_count(companion, CounterKind::Impostor), 1);
    assert_eq!(game.counter_count(other, CounterKind::Impostor), 0);

    cast(&mut game, removal, Some(Target::Card(victim)));
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    let mut declined = game.clone();
    act(&mut declined, Action::FinishTargets);
    settle(&mut declined);
    assert_eq!(declined.object_view(victim).zone, Zone::Exile);
    assert_eq!(declined.object_view(returnable).zone, Zone::Graveyard);

    target(&mut game, returnable);
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Exile);
    assert!(game.on_battlefield(returnable));
    assert_eq!(game.counter_count(returnable, CounterKind::Impostor), 0);
}

#[test]
fn polygraph_orb_keeps_the_library_choice_private_and_requires_all_activation_costs() {
    let player = Player::P1;
    let opponent = player.other();
    let mut game = scenario(player);
    let orb = put(&mut game, player, "Polygraph Orb", Zone::Hand);
    let four = ["Forest", "Hill Giant", "Opt", "Shock"]
        .map(|name| put(&mut game, player, name, Zone::Library));
    let discard = put(&mut game, opponent, "Swamp", Zone::Hand);
    let creature = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut game, opponent, "Candy Trail", Zone::Battlefield);
    cast(&mut game, orb, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert!(game.observe(player).library_selection.is_some());
    assert!(game.observe(opponent).library_selection.is_none());
    reject(&mut game, Action::FinishLibrarySelection);
    for card in [four[0], four[2]] {
        act(&mut game, Action::SelectLibrary { card });
    }
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    assert_eq!(life(&game, player), 18);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 0);
    for card in [four[0], four[2]] {
        assert_eq!(game.object_view(card).zone, Zone::Hand);
    }
    for card in [four[1], four[3]] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }

    let mut no_mana = game.clone();
    no_mana.state.players[player.index()].mana = [0; 6];
    reject(&mut no_mana, activation(orb));
    act(&mut game, activation(orb));
    assert!(!game.object_view(orb).tapped);
    reject(&mut game, Action::FinishEvidence);
    evidence(&mut game, &[four[1]]);
    assert!(!game.object_view(orb).tapped);
    pay(&mut game);
    assert!(game.object_view(orb).tapped);
    assert_eq!(game.object_view(four[1]).zone, Zone::Exile);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(opponent));
    let object = game.object_ref(artifact);
    reject(&mut game, Action::Sacrifice { object });

    let mut discarded = game.clone();
    act(&mut discarded, Action::Discard { card: discard });
    assert_eq!(discarded.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(life(&discarded, opponent), 20);
    let mut sacrificed = game.clone();
    let object = sacrificed.object_ref(creature);
    act(&mut sacrificed, Action::Sacrifice { object });
    assert_eq!(sacrificed.object_view(creature).zone, Zone::Graveyard);
    assert_eq!(life(&sacrificed, opponent), 20);
    act(&mut game, Action::DeclineEffectPayment);
    assert_eq!(life(&game, opponent), 17);
    assert!(game.on_battlefield(creature) && game.on_battlefield(artifact));
}

#[test]
fn rampage_counts_prior_and_current_artifact_arrivals_and_returns_the_exact_creature() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Anzrag's Rampage", Zone::Hand);
    let prior = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let controlled = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let destroyed = put(&mut game, opponent, "Candy Trail", Zone::Battlefield);
    let indestructible = put(&mut game, opponent, "Candy Trail", Zone::Battlefield);
    game.state.objects[indestructible as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    act(&mut game, activation(prior));
    pay(&mut game);
    settle(&mut game);
    let left = put(&mut game, player, "Forest", Zone::Library);
    let land = put(&mut game, player, "Island", Zone::Library);
    let creature = put(&mut game, player, "Hill Giant", Zone::Library);

    cast(&mut game, spell, None);
    pass_pair(&mut game);
    assert!(game.on_battlefield(controlled) && game.on_battlefield(indestructible));
    assert_eq!(game.object_view(destroyed).zone, Zone::Graveyard);
    assert_eq!(game.object_view(left).zone, Zone::Library);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    assert_eq!(game.object_view(creature).zone, Zone::Exile);
    let object = game.object_ref(land);
    reject(&mut game, Action::SelectMilled { object });
    let mut declined = game.clone();
    act(&mut declined, Action::FinishMilledSelection);
    assert_eq!(declined.object_view(creature).zone, Zone::Exile);

    let object = game.object_ref(creature);
    act(&mut game, Action::SelectMilled { object });
    settle(&mut game);
    assert!(game.on_battlefield(creature));
    assert!(game.keywords(creature).contains(Keywords::HASTE));
    enter_step(&mut game, Step::End);
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Hand);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
}

#[test]
fn connecting_the_dots_exiles_each_attack_result_blind_and_only_retrieves_its_own() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Connecting the Dots", Zone::Hand);
    let unlinked = put(&mut game, player, "Connecting the Dots", Zone::Battlefield);
    let attacker = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    let linked = put(&mut game, player, "Opt", Zone::Library);
    let second = put(&mut game, player, "Swamp", Zone::Library);
    cast(&mut game, source, None);
    settle(&mut game);
    begin_attack(&mut game, attacker);
    let id = game
        .state
        .ordering_triggers
        .iter()
        .find(|item| item.source.is_some_and(|source| source.id == unlinked))
        .unwrap()
        .id;
    act(&mut game, Action::PutTrigger { id });
    settle(&mut game);
    for card in [linked, second] {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
        assert!(game.exiled_face_down(game.object_ref(card)));
        for viewer in [player, player.other()] {
            assert!(
                game.observe(viewer)
                    .objects
                    .iter()
                    .all(|object| object.id != card)
            );
        }
    }

    let mut empty = game.clone();
    act(&mut empty, activation(source));
    pay(&mut empty);
    settle(&mut empty);
    assert_eq!(empty.object_view(second).zone, Zone::Hand);
    assert_eq!(empty.object_view(linked).zone, Zone::Exile);

    let discard = put(&mut game, player, "Forest", Zone::Hand);
    act(&mut game, activation(source));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(game.object_view(second).zone, Zone::Hand);
    assert_eq!(game.object_view(linked).zone, Zone::Exile);
}

#[test]
fn expedited_inheritance_gives_the_lethally_damaged_creatures_controller_the_offer() {
    let player = Player::P0;
    let beneficiary = player.other();
    let mut game = scenario(player);
    let enchantment = put(&mut game, player, "Expedited Inheritance", Zone::Hand);
    let victim = put(&mut game, beneficiary, "Bear Cub", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    let spell = put(&mut game, beneficiary, "Think Twice", Zone::Library);
    let land = put(&mut game, beneficiary, "Island", Zone::Library);
    cast(&mut game, enchantment, None);
    settle(&mut game);
    cast(&mut game, shock, Some(Target::Card(victim)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(beneficiary));
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.object_view(spell).zone, Zone::Library);

    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    for card in [spell, land] {
        assert!(
            game.observe(beneficiary)
                .exile_permissions
                .iter()
                .any(|permission| permission.object == game.object_ref(card)
                    && permission.player == beneficiary
                    && permission.expiry == ExilePlayExpiry::EndOfNextTurn
                    && !permission.without_paying_mana_cost)
        );
    }
    if game.actor() != Some(beneficiary) {
        act(&mut game, Action::Pass);
    }
    act(&mut game, Action::AnnounceExile { card: spell });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(
        game.state.players[beneficiary.index()].cards_drawn_this_turn,
        1
    );
}

#[test]
fn harried_dronesmith_makes_a_hasty_thopter_on_its_own_turn_and_sacrifices_it() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Harried Dronesmith", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    let mut opposing = game.clone();
    opposing.state.active = player.other();
    opposing.state.priority = player.other();
    enter_step(&mut opposing, Step::BeginCombat);
    assert!(opposing.state.stack.is_empty());
    enter_step(&mut game, Step::BeginCombat);
    settle(&mut game);
    let [token] = tokens(&game, player, THOPTER).try_into().unwrap();
    assert_eq!(stats(&game, token), (1, 1));
    assert_eq!(game.object_view(token).kinds & 17, 17);
    assert_eq!(game.object_view(token).colors, 0);
    assert!(
        game.keywords(token)
            .contains(Keywords::FLYING.union(Keywords::HASTE))
    );

    let removal = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, removal, Some(Target::Card(source)));
    settle(&mut game);
    enter_step(&mut game, Step::End);
    settle(&mut game);
    assert!(!game.on_battlefield(token));
    assert_eq!(
        game.state.players[player.index()].permanents_sacrificed_this_turn,
        1
    );
}

#[test]
fn incinerator_of_the_guilty_spends_declared_evidence_and_damages_only_eligible_permanents() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(
        &mut game,
        player,
        "Incinerator of the Guilty",
        Zone::Battlefield,
    );
    let fuel = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let creature = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
    let walker = put(
        &mut game,
        opponent,
        "Chandra, Flameshaper",
        Zone::Battlefield,
    );
    game.add_counter(walker, CounterKind::Loyalty, 5);
    let artifact = put(&mut game, opponent, "Candy Trail", Zone::Battlefield);
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);

    assert!(
        game.keywords(source)
            .contains(Keywords::FLYING.union(Keywords::TRAMPLE))
    );
    begin_attack(&mut game, source);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(life(&game, opponent), 14);
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert!(declined.on_battlefield(creature));
    act(&mut game, Action::ChooseOptional { accept: true });
    reject(&mut game, Action::ChooseX { value: 5 });
    act(&mut game, Action::ChooseX { value: 2 });
    evidence(&mut game, &[fuel]);
    assert!(game.on_battlefield(creature));
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Graveyard);
    assert_eq!(game.counter_count(walker, CounterKind::Loyalty), 3);
    assert_eq!(game.object_view(artifact).damage, 0);
    assert_eq!(game.object_view(own).damage, 0);
    assert_eq!(life(&game, opponent), 14);
}

#[test]
fn krenko_counts_actual_artifact_graveyard_arrivals_and_grows_current_goblins() {
    let player = Player::P1;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Krenko, Baron of Tin Street", Zone::Hand);
    let goblin = put(&mut game, player, "Goblin Piker", Zone::Battlefield);
    let artifact = put(&mut game, player, "Candy Trail", Zone::Battlefield);
    let other = put(&mut game, player, "Bear Cub", Zone::Battlefield);
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(game.keywords(source).contains(Keywords::HASTE));
    let mut exiled = game.clone();
    put(
        &mut exiled,
        player.other(),
        "Rest in Peace",
        Zone::Battlefield,
    );
    act(&mut exiled, activation(source));
    additional(&mut exiled, artifact);
    settle(&mut exiled);
    assert_eq!(exiled.object_view(artifact).zone, Zone::Exile);
    assert!(tokens(&exiled, player, GOBLIN).is_empty());
    assert_eq!(exiled.counter_count(goblin, CounterKind::PlusOne), 1);

    act(&mut game, activation(source));
    let object = game.object_ref(other);
    reject(&mut game, Action::ChooseAdditionalCost { object });
    additional(&mut game, artifact);
    assert!(game.object_view(source).tapped);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineResolutionMana);
    settle(&mut declined);
    assert!(tokens(&declined, player, GOBLIN).is_empty());
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    settle(&mut game);
    let [new] = tokens(&game, player, GOBLIN).try_into().unwrap();
    assert!(game.keywords(new).contains(Keywords::HASTE));
    assert_eq!(game.object_view(new).colors, 8);
    for recipient in [source, goblin, new] {
        assert_eq!(game.counter_count(recipient, CounterKind::PlusOne), 1);
    }
    assert_eq!(game.counter_count(other, CounterKind::PlusOne), 0);
    enter_step(&mut game, Step::End);
    pass_pair(&mut game);
    assert!(!game.keywords(new).contains(Keywords::HASTE));
}

#[test]
fn krenkos_buzzcrusher_chooses_untargeted_lands_and_searches_only_for_destroyed_ones() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Krenko's Buzzcrusher", Zone::Hand);
    let own = put(&mut game, player, "Evolving Wilds", Zone::Battlefield);
    let protected = put(&mut game, opponent, "Evolving Wilds", Zone::Battlefield);
    let illegal = put(&mut game, opponent, "Forest", Zone::Battlefield);
    let found = put(&mut game, player, "Island", Zone::Library);
    game.state.objects[own as usize].gained_keywords = Keywords::HEXPROOF;
    game.state.objects[protected as usize].gained_keywords = Keywords::INDESTRUCTIBLE;
    cast(&mut game, source, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let object = game.object_ref(illegal);
    reject(&mut game, Action::ChooseEffectObject { object });
    pick(&mut game, own);
    pick(&mut game, protected);
    act(&mut game, Action::FinishEffectChoice);
    assert_eq!(game.object_view(own).zone, Zone::Graveyard);
    assert!(game.on_battlefield(protected));
    assert_eq!(game.actor(), Some(player));
    act(&mut game, Action::ChooseOptional { accept: true });
    assert!(game.observe(opponent).search_choice.is_none());
    act(&mut game, Action::ChooseSearch { card: found });
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert!(game.object_view(found).tapped);
    assert!(game.on_battlefield(found));
    assert!(
        game.keywords(source)
            .contains(Keywords::FLYING.union(Keywords::TRAMPLE))
    );
}

#[test]
fn lamplight_phoenix_excludes_itself_from_evidence_and_returns_tapped_to_its_owner() {
    let player = Player::P0;
    let owner = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, owner, "Lamplight Phoenix", Zone::Battlefield);
    assert!(game.keywords(source).contains(Keywords::FLYING));
    game.record_control(game.object_ref(source), player, None);
    let material = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let wrong_owner = put(&mut game, owner, "Hill Giant", Zone::Graveyard);
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, murder, Some(Target::Card(source)));
    pass_pair(&mut game);
    let death = game.object_ref(source);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(player));
    reject(&mut game, Action::ChooseEvidence { object: death });
    let object = game.object_ref(wrong_owner);
    reject(&mut game, Action::ChooseEvidence { object });
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEvidence);
    assert_eq!(declined.object_ref(source), death);
    assert_eq!(declined.object_view(material).zone, Zone::Graveyard);

    evidence(&mut game, &[material]);
    settle(&mut game);
    assert_eq!(game.object_view(material).zone, Zone::Exile);
    let returned = game.object_view(source);
    assert_eq!(returned.zone, Zone::Battlefield);
    assert_eq!(returned.controller, owner);
    assert!(returned.tapped);
    assert_ne!(returned.reference, death);
}

#[test]
fn surveillance_monitor_creates_one_thopter_for_each_actual_evidence_payment() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Surveillance Monitor", Zone::Hand);
    let orb = put(&mut game, player, "Polygraph Orb", Zone::Battlefield);
    let mut short = game.clone();
    cast(&mut short, source, None);
    settle(&mut short);
    assert!(tokens(&short, player, THOPTER).is_empty());

    let first = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let second = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.observe(player).evidence.unwrap().threshold, 4);
    let mut declined = game.clone();
    act(&mut declined, Action::DeclineEvidence);
    assert!(tokens(&declined, player, THOPTER).is_empty());
    evidence(&mut game, &[first]);
    settle(&mut game);
    let [token] = tokens(&game, player, THOPTER).try_into().unwrap();
    assert_eq!(stats(&game, token), (1, 1));
    assert!(game.keywords(token).contains(Keywords::FLYING));

    act(&mut game, activation(orb));
    evidence(&mut game, &[second]);
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(tokens(&game, player, THOPTER).len(), 2);
    pass_pair(&mut game);
    act(&mut game, Action::DeclineEffectPayment);
    assert_eq!(life(&game, player.other()), 17);
}
