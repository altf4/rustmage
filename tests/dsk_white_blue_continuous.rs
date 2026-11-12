//! Card-level timing, protection, and continuous effects for Duskmourn.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const PLAYER: Player = Player::P0;
const OPPONENT: Player = Player::P1;
const GLIMMER: Card = Card::from_catalogue_id(5294);

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn end_turn(game: &mut Game) {
    let active = game.state.active;
    game.begin_step(Step::End);
    game.prepare_triggers();
    settle(game);
    pass_pair(game);
    assert_eq!(game.state.active, active.other());
}

fn start_attacks(game: &mut Game) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

#[test]
fn domination_changes_control_weakens_and_blanks_only_its_attached_creature() {
    let mut game = scenario(PLAYER);
    let aura = put(&mut game, PLAYER, "Duskmourn's Domination", Zone::Hand);
    let creature = put(&mut game, OPPONENT, "Spectral Sailor", Zone::Battlefield);
    let other = put(&mut game, OPPONENT, "Serra Angel", Zone::Battlefield);
    let jump = put(&mut game, PLAYER, "Jump Scare", Zone::Hand);
    let remove = put(&mut game, PLAYER, "Disenchant", Zone::Hand);
    game.add_counter(creature, CounterKind::Lifelink, 1);
    let original = game.object_ref(creature);

    cast(&mut game, aura, Some(Target::Card(creature)));
    settle(&mut game);
    let enchanted = game.object_view(creature);
    assert_eq!(game.object_view(aura).attached_to, Some(original));
    assert_eq!(
        (enchanted.reference, enchanted.controller),
        (original, PLAYER)
    );
    assert_eq!(stats(&game, creature), (-2, 1));
    assert!(!enchanted.keywords.contains(Keywords::FLYING));
    assert!(!enchanted.keywords.contains(Keywords::LIFELINK));
    assert!(game.object_view(other).keywords.contains(Keywords::FLYING));
    reject(&mut game, activate(creature, 0));

    cast(&mut game, jump, Some(Target::Card(creature)));
    settle(&mut game);
    assert_eq!(stats(&game, creature), (0, 3));
    assert!(
        game.object_view(creature)
            .keywords
            .contains(Keywords::FLYING)
    );
    assert!(
        !game
            .object_view(creature)
            .keywords
            .contains(Keywords::LIFELINK)
    );

    cast(&mut game, remove, Some(Target::Card(aura)));
    settle(&mut game);
    let freed = game.object_view(creature);
    assert_eq!((freed.reference, freed.controller), (original, OPPONENT));
    assert_eq!(stats(&game, creature), (3, 3));
    assert!(
        freed
            .keywords
            .contains(Keywords::FLYING.union(Keywords::LIFELINK))
    );
}

#[test]
fn fear_of_falling_uses_the_planeswalkers_defender_and_expires_on_the_next_own_turn() {
    let mut game = scenario(PLAYER);
    let fear = put(&mut game, PLAYER, "Fear of Falling", Zone::Battlefield);
    let own = put(&mut game, PLAYER, "Bear Cub", Zone::Battlefield);
    let opposing = put(&mut game, OPPONENT, "Serra Angel", Zone::Battlefield);
    let walker = put(
        &mut game,
        OPPONENT,
        "Liliana, Dreadhorde General",
        Zone::Battlefield,
    );
    let jump = put(&mut game, OPPONENT, "Jump Scare", Zone::Hand);
    game.add_counter(walker, CounterKind::Loyalty, 6);
    assert!(game.object_view(fear).keywords.contains(Keywords::FLYING));
    start_attacks(&mut game);
    let defender = game.object_ref(walker);
    act(
        &mut game,
        Action::AttackPlaneswalker {
            card: fear,
            defender,
        },
    );
    act(&mut game, Action::FinishAttackers);
    reject(&mut game, target(own));
    reject(&mut game, target(walker));
    act(&mut game, target(opposing));

    let mut changed = game.clone();
    changed.change_controller(changed.object_ref(opposing), PLAYER);
    settle(&mut changed);
    assert_eq!(stats(&changed, opposing), (4, 4));
    assert!(
        changed
            .object_view(opposing)
            .keywords
            .contains(Keywords::FLYING)
    );

    settle(&mut game);
    assert_eq!(stats(&game, opposing), (2, 4));
    assert!(
        !game
            .object_view(opposing)
            .keywords
            .contains(Keywords::FLYING)
    );
    end_turn(&mut game);
    assert_eq!(stats(&game, opposing), (2, 4));
    assert!(
        !game
            .object_view(opposing)
            .keywords
            .contains(Keywords::FLYING)
    );

    let mut later_grant = game.clone();
    later_grant.state.players[OPPONENT.index()].mana[0] = 1;
    cast(&mut later_grant, jump, Some(Target::Card(opposing)));
    settle(&mut later_grant);
    assert!(
        later_grant
            .object_view(opposing)
            .keywords
            .contains(Keywords::FLYING)
    );
    assert_eq!(stats(&later_grant, opposing), (4, 6));
    end_turn(&mut game);
    assert_eq!(stats(&game, opposing), (4, 4));
    assert!(
        game.object_view(opposing)
            .keywords
            .contains(Keywords::FLYING)
    );
}

#[test]
fn jump_scare_preserves_other_types_and_does_not_create_an_eerie_entry() {
    let mut game = scenario(PLAYER);
    let spell = put(&mut game, PLAYER, "Jump Scare", Zone::Hand);
    let creature = put(&mut game, PLAYER, "Gingerbrute", Zone::Battlefield);
    let observer = put(&mut game, PLAYER, "Optimistic Scavenger", Zone::Battlefield);
    let before = game.object_view(creature);
    cast(&mut game, spell, Some(Target::Card(creature)));
    settle(&mut game);
    let changed = game.object_view(creature);
    assert_eq!(changed.reference, before.reference);
    assert_eq!(changed.kinds, before.kinds | 32);
    assert!(changed.has_subtype("Food") && changed.has_subtype("Golem"));
    assert!(changed.has_subtype("Horror"));
    assert!(changed.keywords.contains(Keywords::FLYING));
    assert_eq!(stats(&game, creature), (3, 3));
    assert_eq!(game.counter_count(observer, CounterKind::PlusOne), 0);

    end_turn(&mut game);
    let expired = game.object_view(creature);
    assert_eq!(expired.kinds, before.kinds);
    assert!(expired.has_subtype("Food") && expired.has_subtype("Golem"));
    assert!(!expired.has_subtype("Horror"));
    assert!(!expired.keywords.contains(Keywords::FLYING));
    assert_eq!(stats(&game, creature), (1, 1));
}

#[test]
fn shardmage_rescues_its_controllers_creature_this_turn_and_keeps_the_stat_bonus() {
    let mut game = scenario(OPPONENT);
    let aura = put(&mut game, PLAYER, "Shardmage's Rescue", Zone::Hand);
    let ally = put(&mut game, PLAYER, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, OPPONENT, "Bear Cub", Zone::Battlefield);
    let shock = put(&mut game, OPPONENT, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(ally)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::AnnounceCast { card: aura });
    reject(&mut game, target(enemy));
    act(&mut game, target(ally));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(
        game.object_view(aura).attached_to,
        Some(game.object_ref(ally))
    );
    assert_eq!(game.object_view(ally).damage, 0);
    assert_eq!(stats(&game, ally), (3, 3));
    assert!(game.object_view(ally).keywords.contains(Keywords::HEXPROOF));

    end_turn(&mut game);
    assert_eq!(stats(&game, ally), (3, 3));
    assert!(!game.object_view(ally).keywords.contains(Keywords::HEXPROOF));
    game.change_controller(game.object_ref(ally), OPPONENT);
    act(&mut game, Action::Pass);
    assert_eq!(game.object_view(aura).zone, Zone::Graveyard);
    assert_eq!(stats(&game, ally), (2, 2));
}

fn kept_leyline(name: &str) -> (Game, ObjectRef) {
    let leyline = Card::from_name(name).unwrap();
    let mut game = Game::new(Config {
        decks: [
            [vec![leyline], vec![Card::Forest; 6]].concat(),
            vec![Card::Forest; 7],
        ],
        seed: 20261023,
        starting_player: PLAYER,
        deck_rules: DeckRules::Freeform,
    })
    .unwrap();
    act(&mut game, Action::Keep);
    act(&mut game, Action::Keep);
    let object = game
        .legal_actions()
        .into_iter()
        .find_map(|action| match action {
            Action::BeginOnBattlefield { object } => Some(object),
            _ => None,
        })
        .unwrap();
    (game, object)
}

fn finish_opening(game: &mut Game) {
    act(game, Action::FinishOpeningHand);
    act(game, Action::FinishOpeningHand);
    assert_eq!(game.state.step, Step::Upkeep);
}

#[test]
fn hope_can_start_on_the_battlefield_and_modifies_each_lifelink_gain() {
    let (mut opening, object) = kept_leyline("Leyline of Hope");
    let mut declined = opening.clone();
    finish_opening(&mut declined);
    assert_eq!(declined.object_view(object.card).zone, Zone::Hand);
    assert!(
        declined
            .observe(OPPONENT)
            .objects
            .iter()
            .all(|row| row.id != object.card)
    );
    act(&mut opening, Action::BeginOnBattlefield { object });
    finish_opening(&mut opening);
    assert_eq!(opening.object_view(object.card).zone, Zone::Battlefield);
    assert_eq!(
        opening.state.players[PLAYER.index()].spells_cast_this_turn,
        0
    );

    let mut game = scenario(PLAYER);
    let leyline = put(&mut game, PLAYER, "Leyline of Hope", Zone::Hand);
    let own = put(&mut game, PLAYER, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, OPPONENT, "Bear Cub", Zone::Battlefield);
    let shock = put(&mut game, OPPONENT, "Shock", Zone::Hand);
    let mountain = put(&mut game, OPPONENT, "Mountain", Zone::Battlefield);
    let hawks: [CardId; 2] =
        std::array::from_fn(|_| put(&mut game, PLAYER, "Healer's Hawk", Zone::Battlefield));
    cast(&mut game, leyline, None);
    settle(&mut game);
    game.state.players[PLAYER.index()].life = 23;
    assert_eq!(stats(&game, own), (2, 2));
    start_attacks(&mut game);
    for card in hawks {
        act(&mut game, Action::Attack { card });
    }
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.state.players[PLAYER.index()].life, 27);
    assert_eq!(game.state.players[OPPONENT.index()].life, 18);
    assert_eq!(stats(&game, own), (4, 4));
    assert_eq!(stats(&game, enemy), (2, 2));

    act(&mut game, Action::Pass);
    act(&mut game, Action::TapMana { card: mountain });
    cast(&mut game, shock, Some(Target::Player(PLAYER)));
    settle(&mut game);
    assert_eq!(game.state.players[PLAYER.index()].life, 25);
    assert_eq!(stats(&game, own), (2, 2));
}

#[test]
fn transformation_adds_the_chosen_type_to_controlled_creatures_and_owned_cards() {
    let (mut game, leyline) = kept_leyline("Leyline of Transformation");
    act(&mut game, Action::BeginOnBattlefield { object: leyline });
    reject(
        &mut game,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::Color(ChoiceColor::Blue),
        },
    );
    let dragon = EntryChoiceValue::CreatureType(CreatureType::from_name("Dragon").unwrap());
    act(&mut game, Action::ChooseEntryValue { value: dragon });
    finish_opening(&mut game);
    assert_eq!(game.object_view(leyline.card).entry_choice, Some(dragon));

    let hand = put(&mut game, PLAYER, "Bear Cub", Zone::Hand);
    let library = put(&mut game, PLAYER, "Bear Cub", Zone::Library);
    let graveyard = put(&mut game, PLAYER, "Bear Cub", Zone::Graveyard);
    let exile = put(&mut game, PLAYER, "Bear Cub", Zone::Exile);
    let foreign = put(&mut game, OPPONENT, "Bear Cub", Zone::Library);
    let taken = put(&mut game, OPPONENT, "Bear Cub", Zone::Battlefield);
    let lost = put(&mut game, PLAYER, "Bear Cub", Zone::Battlefield);
    game.change_controller(game.object_ref(taken), PLAYER);
    game.change_controller(game.object_ref(lost), OPPONENT);
    for card in [hand, library, graveyard, exile, taken] {
        let view = game.object_view(card);
        assert!(view.has_subtype("Bear") && view.has_subtype("Dragon"));
    }
    for card in [foreign, lost] {
        assert!(!game.object_view(card).has_subtype("Dragon"));
    }
    assert!(
        game.observe(OPPONENT)
            .objects
            .iter()
            .all(|row| row.id != hand && row.id != library)
    );

    game.state.players[PLAYER.index()].mana = [8; 6];
    game.begin_step(Step::Main1);
    game.prepare_triggers();
    cast(&mut game, hand, None);
    assert_eq!(game.object_view(hand).zone, Zone::Stack);
    assert!(game.object_view(hand).has_subtype("Dragon"));
    let mut stolen_spell = game.clone();
    stolen_spell.change_controller(stolen_spell.object_ref(hand), OPPONENT);
    assert!(!stolen_spell.object_view(hand).has_subtype("Dragon"));
    settle(&mut game);
    assert!(game.object_view(hand).has_subtype("Dragon"));

    let tutor = put(
        &mut game,
        PLAYER,
        "Maelstrom of the Spirit Dragon",
        Zone::Battlefield,
    );
    act(&mut game, activate(tutor, 2));
    pay(&mut game);
    pass_pair(&mut game);
    let choices = game.observe(PLAYER).search_choice.unwrap();
    assert!(
        choices
            .cards
            .iter()
            .any(|row| row.id == library && row.has_subtype("Dragon"))
    );
    assert!(game.observe(OPPONENT).search_choice.is_none());
    reject(&mut game, Action::ChooseSearch { card: foreign });
    act(&mut game, Action::ChooseSearch { card: library });
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.object_view(library).zone, Zone::Hand);

    let disenchant = put(&mut game, PLAYER, "Disenchant", Zone::Hand);
    cast(&mut game, disenchant, Some(Target::Card(leyline.card)));
    settle(&mut game);
    for card in [hand, library, graveyard, exile, taken] {
        assert!(!game.object_view(card).has_subtype("Dragon"));
    }
}

#[test]
fn possessed_goat_pays_a_discard_once_and_keeps_its_original_color_and_type() {
    let mut game = scenario(PLAYER);
    let goat = put(&mut game, PLAYER, "Possessed Goat", Zone::Battlefield);
    let discard = put(&mut game, PLAYER, "Island", Zone::Hand);
    put(&mut game, PLAYER, "Forest", Zone::Hand);
    let tidebinder = put(&mut game, OPPONENT, "Tishana's Tidebinder", Zone::Hand);
    let initial = game.object_ref(goat);
    let activation = activate(goat, 0);
    act(&mut game, activation.clone());
    let payment = game.object_ref(discard);
    act(&mut game, Action::ChooseAdditionalCost { object: payment });
    pay(&mut game);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    let ability = game.state.stack.last().unwrap().id;
    reject(&mut game, activation.clone());

    let mut countered = game.clone();
    act(&mut countered, Action::Pass);
    cast(&mut countered, tidebinder, None);
    pass_pair(&mut countered);
    act(
        &mut countered,
        Action::ChooseTarget {
            target: Target::Stack(ability),
        },
    );
    settle(&mut countered);
    assert_eq!(stats(&countered, goat), (1, 1));
    assert!(!countered.object_view(goat).has_subtype("Demon"));
    countered.move_object(tidebinder, Zone::Graveyard);
    reject(&mut countered, activation.clone());

    settle(&mut game);
    let demon = game.object_view(goat);
    assert_eq!(demon.reference, initial);
    assert_eq!(demon.colors, 1 | 4);
    assert!(demon.has_subtype("Goat") && demon.has_subtype("Demon"));
    assert_eq!(game.counter_count(goat, CounterKind::PlusOne), 3);
    assert_eq!(stats(&game, goat), (4, 4));
    end_turn(&mut game);
    assert_eq!(game.object_view(goat).colors, 1 | 4);
    assert!(game.object_view(goat).has_subtype("Demon"));

    game.move_object(goat, Zone::Exile);
    game.move_object(goat, Zone::Battlefield);
    let returned = game.object_view(goat);
    assert_ne!(returned.reference, initial);
    assert_eq!(returned.colors, 1);
    assert!(returned.has_subtype("Goat") && !returned.has_subtype("Demon"));
    assert_eq!(stats(&game, goat), (1, 1));
    game.state.players[PLAYER.index()].mana[5] = 3;
    act(&mut game, Action::Pass);
    assert!(game.legal_actions().contains(&activation));
}

#[test]
fn wandering_rescuer_convoke_can_protect_its_tapped_payment_and_it_strikes_twice() {
    let mut game = scenario(OPPONENT);
    let rescuer = put(&mut game, PLAYER, "The Wandering Rescuer", Zone::Hand);
    let convoker = put(&mut game, PLAYER, "Healer's Hawk", Zone::Battlefield);
    let untapped = put(&mut game, PLAYER, "Bear Cub", Zone::Battlefield);
    let noncreature = put(&mut game, PLAYER, "Forest", Zone::Battlefield);
    let enemy = put(&mut game, OPPONENT, "Bear Cub", Zone::Battlefield);
    let shock = put(&mut game, OPPONENT, "Shock", Zone::Hand);
    for tapped in [noncreature, enemy] {
        game.state.objects[tapped as usize].tapped = true;
    }
    game.state.objects[convoker as usize].summoning_sick = true;
    game.state.players[PLAYER.index()].mana = [1, 0, 0, 0, 0, 3];
    cast(&mut game, shock, Some(Target::Card(convoker)));
    act(&mut game, Action::Pass);
    act(&mut game, Action::AnnounceCast { card: rescuer });
    act(&mut game, Action::BeginConvoke);
    let object = game.object_ref(convoker);
    act(
        &mut game,
        Action::ChooseConvoker {
            object,
            contribution: ConvokeContribution::White,
        },
    );
    assert!(!game.object_view(convoker).tapped);
    pay(&mut game);
    assert!(game.object_view(convoker).tapped);
    assert!(
        !game
            .object_view(convoker)
            .keywords
            .contains(Keywords::HEXPROOF)
    );
    settle(&mut game);
    assert_eq!(game.object_view(convoker).damage, 0);
    assert!(
        game.object_view(convoker)
            .keywords
            .contains(Keywords::HEXPROOF)
    );
    game.state.objects[rescuer as usize].tapped = true;
    for unprotected in [untapped, noncreature, enemy, rescuer] {
        assert!(
            !game
                .object_view(unprotected)
                .keywords
                .contains(Keywords::HEXPROOF)
        );
    }
    assert!(
        game.object_view(rescuer)
            .keywords
            .contains(Keywords::DOUBLE_STRIKE)
    );

    end_turn(&mut game);
    assert!(!game.object_view(convoker).tapped);
    assert!(
        !game
            .object_view(convoker)
            .keywords
            .contains(Keywords::HEXPROOF)
    );
    start_attacks(&mut game);
    act(&mut game, Action::Attack { card: rescuer });
    act(&mut game, Action::FinishAttackers);
    pass_pair(&mut game);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::FirstStrikeDamage);
    assert_eq!(game.state.players[OPPONENT.index()].life, 17);
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::CombatDamage);
    assert_eq!(game.state.players[OPPONENT.index()].life, 14);
}

const BEAST: Card = Card::from_catalogue_id(5674);

#[test]
fn toby_creates_the_restricted_beast_and_only_flies_its_own_creature_tokens_at_four() {
    let mut game = scenario(PLAYER);
    let toby = put(&mut game, PLAYER, "Toby, Beastie Befriender", Zone::Hand);
    cast(&mut game, toby, None);
    settle(&mut game);
    let beast = game.controlled_token_ids(PLAYER).next().unwrap();
    let token = game.object_view(beast);
    assert_eq!(token.card, BEAST);
    assert_eq!(token.name, "Beast Token");
    assert_eq!(stats(&game, beast), (4, 4));
    assert_eq!(token.colors, 1);
    assert!(token.has_subtype("Beast"));
    assert!(!token.keywords.contains(Keywords::FLYING));

    let mut attack = game.clone();
    attack.state.objects[toby as usize].tapped = true;
    attack.state.objects[beast as usize].summoning_sick = false;
    start_attacks(&mut attack);
    let attack_beast = Action::Attack { card: beast };
    reject(&mut attack, attack_beast.clone());
    let companion = put(&mut attack, PLAYER, "Bear Cub", Zone::Battlefield);
    act(&mut attack, attack_beast);
    reject(&mut attack, Action::FinishAttackers);
    act(&mut attack, Action::Attack { card: companion });
    act(&mut attack, Action::FinishAttackers);
    assert_eq!(attack.state.attackers.len(), 2);

    let mut block = game.clone();
    block.state.objects[toby as usize].tapped = true;
    let first = put(&mut block, OPPONENT, "Bear Cub", Zone::Battlefield);
    let second = put(&mut block, OPPONENT, "Bear Cub", Zone::Battlefield);
    end_turn(&mut block);
    start_attacks(&mut block);
    for card in [first, second] {
        act(&mut block, Action::Attack { card });
    }
    act(&mut block, Action::FinishAttackers);
    pass_pair(&mut block);
    let block_beast = Action::Block {
        blocker: beast,
        attacker: first,
    };
    reject(&mut block, block_beast.clone());
    let companion = put(&mut block, PLAYER, "Bear Cub", Zone::Battlefield);
    act(&mut block, block_beast);
    reject(&mut block, Action::FinishBlockers);
    act(
        &mut block,
        Action::Block {
            blocker: companion,
            attacker: second,
        },
    );
    act(&mut block, Action::FinishBlockers);

    let foreign = game.state.objects.len() as CardId;
    game.create_tokens(OPPONENT, GLIMMER, 1, false);
    game.create_tokens(PLAYER, GLIMMER, 2, false);
    game.create_tokens(PLAYER, Card::from_catalogue_id(5302), 1, false);
    assert!(!game.object_view(beast).keywords.contains(Keywords::FLYING));
    let fourth = game.state.objects.len() as CardId;
    game.create_tokens(PLAYER, GLIMMER, 1, false);
    for creature in game
        .controlled_token_ids(PLAYER)
        .filter(|&id| game.object_view(id).is_creature())
    {
        assert!(
            game.object_view(creature)
                .keywords
                .contains(Keywords::FLYING)
        );
    }
    for grounded in [foreign, toby] {
        assert!(
            !game
                .object_view(grounded)
                .keywords
                .contains(Keywords::FLYING)
        );
    }
    game.move_object(fourth, Zone::Graveyard);
    assert!(!game.object_view(beast).keywords.contains(Keywords::FLYING));
}
