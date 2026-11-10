//! Paid Spider-Man rules scenarios for collector-group members 001–048.
mod support;

use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::cast_costs::CastCostClause;
use rustmage_core::engine::*;
use support::*;

const P: Player = Player::P0;
const O: Player = Player::P1;
const WHITE: Mana = [1, 0, 0, 0, 0, 0];
const BLUE: Mana = [0, 1, 0, 0, 0, 0];

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn target_action(target: Target) -> Action {
    Action::ChooseTarget { target }
}

fn target(game: &mut Game, subject: Target) {
    act(game, target_action(subject));
}

fn finish_targets(game: &mut Game) {
    if game.legal_actions().contains(&Action::FinishTargets) {
        act(game, Action::FinishTargets);
    }
}

fn mode(game: &mut Game, index: u16) {
    act(game, Action::ChooseMode { mode: index });
}

fn announce_activation(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn activate(game: &mut Game, card: CardId, ability: u16, chosen: Option<Target>) {
    if let Some(action) = game.legal_actions().into_iter().find(|action| {
        matches!(action, Action::Activate { card: id, ability: slot, target, .. }
            if *id == card && *slot == ability && *target == chosen)
    }) {
        act(game, action);
        return;
    }
    act(game, announce_activation(card, ability));
    if let Some(chosen) = chosen {
        target(game, chosen);
    }
    finish_targets(game);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
}

fn normal_cast_available(game: &Game, card: CardId) -> bool {
    game.legal_actions().into_iter().any(|action| {
        matches!(action, Action::AnnounceCast { card: id } | Action::Cast { card: id, .. }
            | Action::BeginCast { card: id, .. } if id == card)
    })
}

fn start_attack(game: &mut Game, attackers: &[CardId]) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    game.state.passes = 0;
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

fn unblocked_damage(game: &mut Game) {
    pass_pair(game);
    act(game, Action::FinishBlockers);
    pass_pair(game);
}

fn order_triggers(game: &mut Game) {
    for _ in 0..8 {
        if !matches!(game.state.pending, Pending::OrderTriggers { .. }) {
            return;
        }
        choose(game, |action| matches!(action, Action::PutTrigger { .. }));
    }
    panic!("eight trigger choices exhausted");
}

fn token(game: &Game, player: Player, subtype: &str) -> CardId {
    game.observe(player)
        .objects
        .into_iter()
        .find(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.is_token()
                && object.has_subtype(subtype)
        })
        .unwrap_or_else(|| panic!("missing {subtype} token"))
        .id
}

fn plus_one(game: &Game, id: CardId) -> u64 {
    game.counter_count(id, CounterKind::PlusOne)
}

#[test]
fn agent_venom_flashes_in_and_observes_only_other_controlled_nontoken_deaths() {
    let mut game = scenario(O);
    let agent = put(&mut game, P, "Agent Venom", Zone::Hand);
    game.state.priority = P;
    cast(&mut game, agent, None);
    settle(&mut game);
    assert!(game.object_view(agent).keywords.contains(Keywords::MENACE));
    let own = put(&mut game, P, "Bear Cub", Zone::Battlefield);
    let opponent = put(&mut game, O, "Bear Cub", Zone::Battlefield);
    let citizen = spawn(
        &mut game,
        P,
        Card::from_catalogue_id(5314),
        Zone::Battlefield,
    );
    for id in [opponent, citizen, own, agent] {
        game.move_object(id, Zone::Graveyard);
        game.prepare_triggers();
        settle(&mut game);
        let expected = u64::from(id == own || id == agent);
        assert_eq!(game.hand_count(P), expected as usize);
        assert_eq!(
            game.state.players[P.index()].life,
            20 - i128::from(expected)
        );
    }
}

#[test]
fn alien_symbiosis_adds_its_aura_traits_and_discards_only_for_its_graveyard_permission() {
    for graveyard in [false, true] {
        let mut game = scenario(P);
        let creature = put(&mut game, O, "Grizzly Bears", Zone::Battlefield);
        let source = put(
            &mut game,
            P,
            "Alien Symbiosis",
            if graveyard {
                Zone::Graveyard
            } else {
                Zone::Hand
            },
        );
        let action = Action::AnnounceGraveyardAdditionalCosts { card: source };
        reject(&mut game, action.clone());
        let discard = put(&mut game, P, "Forest", Zone::Hand);
        if graveyard {
            act(&mut game, action);
            target(&mut game, Target::Card(creature));
            finish_targets(&mut game);
            for _ in 0..8 {
                match game.state.pending {
                    Pending::PayCosts { .. } => pay(&mut game),
                    Pending::CastCosts { .. } => {
                        let costs = game.observe(P).cast_costs.unwrap();
                        let clause = costs.clauses.iter().position(|row| !row.paid).unwrap() as u16;
                        act(&mut game, Action::ChooseCastCost { clause });
                    }
                    Pending::AdditionalCost { .. } => {
                        let object = game.object_ref(discard);
                        act(&mut game, Action::ChooseAdditionalCost { object });
                    }
                    Pending::Priority => break,
                    ref other => panic!("unexpected graveyard payment phase {other:?}"),
                }
            }
        } else {
            cast(&mut game, source, Some(Target::Card(creature)));
        }
        settle(&mut game);
        assert_eq!(
            game.object_view(discard).zone,
            if graveyard {
                Zone::Graveyard
            } else {
                Zone::Hand
            },
        );
        assert_eq!(
            game.object_view(source).attached_to,
            Some(game.object_ref(creature))
        );
        let enchanted = game.object_view(creature);
        assert!(enchanted.has_subtype("Bear"));
        assert!(enchanted.has_subtype("Symbiote"));
        assert!(enchanted.keywords.contains(Keywords::MENACE));
        assert_eq!(stats(&game, creature), (3, 3));
        game.move_object(source, Zone::Graveyard);
        assert!(!game.object_view(creature).has_subtype("Symbiote"));
        assert_eq!(stats(&game, creature), (2, 2));
    }
}

#[test]
fn amazing_acrobatics_chooses_each_mode_and_keeps_legal_targets_across_groups() {
    for selected in [0, 1, 2] {
        let mut game = scenario(O);
        let spell = put(&mut game, O, "Divination", Zone::Hand);
        let first = put(&mut game, O, "Bear Cub", Zone::Battlefield);
        let second = put(&mut game, O, "Grizzly Bears", Zone::Battlefield);
        let acrobatics = put(&mut game, P, "Amazing Acrobatics", Zone::Hand);
        cast(&mut game, spell, None);
        act(&mut game, Action::Pass);
        act(&mut game, Action::AnnounceCast { card: acrobatics });
        mode(&mut game, selected);
        if selected != 1 {
            target(&mut game, Target::Spell(spell));
            finish_targets(&mut game);
        }
        if selected != 0 {
            target(&mut game, Target::Card(first));
            target(&mut game, Target::Card(second));
            finish_targets(&mut game);
        }
        pay(&mut game);
        if selected == 2 {
            game.move_object(second, Zone::Exile);
        }
        settle(&mut game);
        assert_eq!(game.hand_count(O), if selected == 1 { 2 } else { 0 });
        assert_eq!(game.object_view(first).tapped, selected != 0);
        assert_eq!(game.object_view(second).tapped, selected == 1);
    }
}

#[test]
fn angry_rabble_checks_cast_mana_value_and_pays_for_two_counters() {
    let mut game = scenario(P);
    let source = put(&mut game, P, "Angry Rabble", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(
        game.object_view(source)
            .keywords
            .contains(Keywords::TRAMPLE)
    );
    for (name, life) in [("Opt", 20), ("Hedron Archive", 19)] {
        let spell = put(&mut game, P, name, Zone::Hand);
        cast(&mut game, spell, None);
        if name == "Opt" {
            pass_pair(&mut game);
            let card = game.top_library_card(P).unwrap();
            act(
                &mut game,
                Action::ArrangeLibrary {
                    card,
                    destination: LibraryDestination::Top,
                },
            );
        }
        settle(&mut game);
        assert_eq!(game.state.players[O.index()].life, life);
    }
    game.state.players[P.index()].mana = [0, 0, 0, 1, 0, 4];
    reject(&mut game, announce_activation(source, 0));
    game.state.players[P.index()].mana[5] = 5;
    activate(&mut game, source, 0, None);
    settle(&mut game);
    assert_eq!(game.state.players[P.index()].mana, [0; 6]);
    assert_eq!(stats(&game, source), (4, 4));
}

#[test]
fn anti_venom_reanimates_only_when_cast_and_turns_damage_into_counters() {
    let mut game = scenario(P);
    let target_card = put(&mut game, P, "Grizzly Bears", Zone::Graveyard);
    let foreign = put(&mut game, O, "Grizzly Bears", Zone::Graveyard);
    let healer = put(&mut game, P, "Anti-Venom, Horrifying Healer", Zone::Hand);
    game.move_object(healer, Zone::Battlefield);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.object_view(target_card).zone, Zone::Graveyard);
    game.move_object(healer, Zone::Hand);
    cast(&mut game, healer, None);
    pass_pair(&mut game);
    reject(&mut game, target_action(Target::Card(foreign)));
    target(&mut game, Target::Card(target_card));
    finish_targets(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(target_card).zone, Zone::Battlefield);
    let shock = put(&mut game, P, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(healer)));
    settle(&mut game);
    assert_eq!(game.object_view(healer).damage, 0);
    assert_eq!(plus_one(&game, healer), 2);
    assert_eq!(stats(&game, healer), (7, 7));
}

#[test]
fn arachne_web_slings_then_peeks_privately_and_taxes_the_chosen_type_for_each_seat() {
    let mut game = scenario(P);
    let body = put(&mut game, P, "Bear Cub", Zone::Battlefield);
    game.state.objects[body as usize].tapped = true;
    let arachne = put(&mut game, P, "Arachne, Psionic Weaver", Zone::Hand);
    let own_shock = put(&mut game, P, "Shock", Zone::Hand);
    let enemy_shock = put(&mut game, O, "Shock", Zone::Hand);
    game.state.players[P.index()].mana = WHITE;
    reject(&mut game, Action::AnnounceCast { card: arachne });
    act(
        &mut game,
        Action::AnnounceWebSlinging {
            card: arachne,
            face: CastFace::Normal,
            cost: ManaCost {
                colored: WHITE,
                generic: 0,
            },
            flash_surcharge: false,
            origin: WebSlingingOrigin::Ordinary,
        },
    );
    act(&mut game, Action::BeginOrderedCosts);
    let costs = game.observe(P).cast_costs.unwrap();
    for (index, row) in costs.clauses.iter().enumerate() {
        let action = Action::ChooseCastCost {
            clause: index as u16,
        };
        if row.clause == CastCostClause::Mana {
            act(&mut game, action);
            pay(&mut game);
        }
    }
    let costs = game.observe(P).cast_costs.unwrap();
    let index = costs
        .clauses
        .iter()
        .position(|row| matches!(row.clause, CastCostClause::WebSlingingReturn(_)))
        .unwrap();
    act(
        &mut game,
        Action::ChooseCastCost {
            clause: index as u16,
        },
    );
    let object = game.object_ref(body);
    act(&mut game, Action::PayWebSlingingReturn { object });
    assert_eq!(game.object_view(body).zone, Zone::Hand);
    assert_eq!(game.state.players[P.index()].mana, [0; 6]);
    pass_pair(&mut game);
    assert!(game.observe(O).entry_value.unwrap().opponent_hand.is_none());
    let looked = game.observe(P).entry_value.unwrap().opponent_hand.unwrap();
    assert_eq!(
        looked.iter().map(|view| view.id).collect::<Vec<_>>(),
        [enemy_shock]
    );
    let value = EntryChoiceValue::CardType(NoncreatureCardType::Instant);
    act(&mut game, Action::ChooseEntryValue { value });
    assert_eq!(game.object_view(arachne).entry_choice, Some(value));
    for (player, id) in [(P, own_shock), (O, enemy_shock)] {
        game.state.priority = player;
        game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
        assert!(!normal_cast_available(&game, id));
        game.state.players[player.index()].mana[5] = 1;
        assert!(normal_cast_available(&game, id));
    }
}

#[test]
fn arana_has_one_attack_trigger_and_a_play_permission_per_modified_combat_hit() {
    let mut game = scenario(P);
    let arana = put(&mut game, P, "Araña, Heart of the Spider", Zone::Hand);
    cast(&mut game, arana, None);
    settle(&mut game);
    let modified = put(&mut game, P, "Bear Cub", Zone::Battlefield);
    let other = put(&mut game, P, "Grizzly Bears", Zone::Battlefield);
    let unmodified = put(&mut game, P, "Bear Cub", Zone::Battlefield);
    game.add_counter(modified, CounterKind::Charge, 1);
    let lands = [
        put(&mut game, P, "Forest", Zone::Library),
        put(&mut game, P, "Mountain", Zone::Library),
    ];
    start_attack(&mut game, &[modified, other]);
    reject(&mut game, target_action(Target::Card(unmodified)));
    target(&mut game, Target::Card(other));
    finish_targets(&mut game);
    settle(&mut game);
    assert_eq!(plus_one(&game, other), 1);
    assert_eq!(plus_one(&game, modified), 0);
    unblocked_damage(&mut game);
    order_triggers(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[O.index()].life, 15);
    for id in lands {
        assert_eq!(game.object_view(id).zone, Zone::Exile);
        assert!(
            game.observe(P)
                .exile_permissions
                .iter()
                .any(|permission| permission.object == game.object_ref(id)
                    && permission.expiry == ExilePlayExpiry::Turn(game.state.turn))
        );
    }
}

#[test]
fn aunt_may_gains_life_for_other_creatures_and_counters_only_the_entering_spider() {
    let mut game = scenario(P);
    let may = put(&mut game, P, "Aunt May", Zone::Hand);
    cast(&mut game, may, None);
    settle(&mut game);
    assert_eq!(game.state.players[P.index()].life, 20);
    let non_spider = put(&mut game, P, "Bear Cub", Zone::Hand);
    let spider = put(&mut game, P, "Treetop Snarespinner", Zone::Hand);
    for id in [non_spider, spider] {
        cast(&mut game, id, None);
        settle(&mut game);
    }
    assert_eq!(game.state.players[P.index()].life, 22);
    assert_eq!(plus_one(&game, non_spider), 0);
    assert_eq!(plus_one(&game, spider), 1);
}

#[test]
fn bagel_shares_with_zero_or_one_target_and_noshes_at_instant_speed() {
    for (ability, include_target, expected_life, expected_counters) in
        [(0, false, 20, 0), (0, true, 20, 1), (1, false, 23, 0)]
    {
        let mut game = scenario(P);
        let food = put(&mut game, P, "Bagel and Schmear", Zone::Hand);
        let creature = put(&mut game, P, "Bear Cub", Zone::Battlefield);
        cast(&mut game, food, None);
        settle(&mut game);
        assert!(game.object_view(food).has_subtype("Food"));
        if ability == 1 {
            game.state.active = O;
            reject(&mut game, announce_activation(food, 0));
        }
        game.state.players[P.index()].mana = if ability == 0 {
            WHITE
        } else {
            [0, 0, 0, 0, 0, 2]
        };
        activate(
            &mut game,
            food,
            ability,
            include_target.then_some(Target::Card(creature)),
        );
        assert_eq!(game.object_view(food).zone, Zone::Graveyard);
        assert_eq!(game.state.players[P.index()].mana, [0; 6]);
        settle(&mut game);
        assert_eq!(game.hand_count(P), 1);
        assert_eq!(game.state.players[P.index()].life, expected_life);
        assert_eq!(plus_one(&game, creature), expected_counters);
    }
}

#[test]
fn beetle_exiles_as_a_graveyard_cost_and_gives_a_counter_and_flying() {
    let mut game = scenario(P);
    let beetle = put(&mut game, P, "Beetle, Legacy Criminal", Zone::Graveyard);
    let creature = put(&mut game, P, "Bear Cub", Zone::Battlefield);
    assert!(
        game.object_view(beetle)
            .card
            .keywords()
            .contains(Keywords::FLYING)
    );
    game.state.active = O;
    reject(&mut game, announce_activation(beetle, 0));
    game.state.active = P;
    game.state.players[P.index()].mana = [0, 1, 0, 0, 0, 1];
    activate(&mut game, beetle, 0, Some(Target::Card(creature)));
    assert_eq!(game.object_view(beetle).zone, Zone::Exile);
    assert_eq!(game.state.players[P.index()].mana, [0; 6]);
    settle(&mut game);
    assert_eq!(plus_one(&game, creature), 1);
    assert!(
        game.object_view(creature)
            .keywords
            .contains(Keywords::FLYING)
    );
}

#[test]
fn behold_returns_distinct_targets_and_leaves_a_duplicate_and_stale_target_behind() {
    let mut game = scenario(P);
    let first = put(&mut game, P, "Bear Cub", Zone::Graveyard);
    let duplicate = put(&mut game, P, "Bear Cub", Zone::Graveyard);
    let stale = put(&mut game, P, "Grizzly Bears", Zone::Graveyard);
    let last = put(&mut game, P, "Hill Giant", Zone::Graveyard);
    let spell = put(&mut game, P, "Behold the Sinister Six!", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: spell });
    target(&mut game, Target::Card(first));
    reject(&mut game, target_action(Target::Card(duplicate)));
    target(&mut game, Target::Card(stale));
    target(&mut game, Target::Card(last));
    finish_targets(&mut game);
    pay(&mut game);
    game.move_object(stale, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Battlefield);
    assert_eq!(game.object_view(last).zone, Zone::Battlefield);
    assert_eq!(game.object_view(duplicate).zone, Zone::Graveyard);
    assert_eq!(game.object_view(stale).zone, Zone::Exile);
}

#[test]
fn carapace_attaches_and_lets_the_creatures_controller_draw_for_modified_creatures() {
    let mut game = scenario(P);
    let first = put(&mut game, P, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, P, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, O, "Bear Cub", Zone::Battlefield);
    let carapace = put(&mut game, P, "Biorganic Carapace", Zone::Hand);
    cast(&mut game, carapace, None);
    pass_pair(&mut game);
    reject(&mut game, target_action(Target::Card(enemy)));
    target(&mut game, Target::Card(first));
    finish_targets(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, first), (4, 4));
    game.state.players[P.index()].mana = [0, 0, 0, 0, 0, 2];
    activate(&mut game, carapace, 0, Some(Target::Card(second)));
    settle(&mut game);
    assert_eq!(stats(&game, first), (2, 2));
    assert_eq!(stats(&game, second), (4, 4));

    game.change_controller(game.object_ref(second), O);
    game.state.active = O;
    game.state.priority = O;
    game.state.objects[second as usize].summoning_sick = false;
    game.add_counter(enemy, CounterKind::Charge, 1);
    game.add_counter(first, CounterKind::Charge, 1);
    start_attack(&mut game, &[second]);
    unblocked_damage(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[P.index()].life, 16);
    assert_eq!(game.hand_count(P), 0);
    assert_eq!(game.hand_count(O), 2);
}

#[test]
fn black_cat_exiles_two_privately_and_keeps_the_paid_play_permission_after_leaving() {
    let mut game = scenario(P);
    let shock = put(&mut game, O, "Shock", Zone::Library);
    let mountain = put(&mut game, O, "Mountain", Zone::Library);
    let source = put(&mut game, P, "Black Cat, Cunning Thief", Zone::Hand);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    reject(&mut game, target_action(Target::Player(P)));
    target(&mut game, Target::Player(O));
    finish_targets(&mut game);
    pass_pair(&mut game);
    let private = game.observe(P).library_selection.unwrap();
    assert_eq!(
        (private.minimum, private.maximum, private.remaining.len()),
        (2, 2, 9)
    );
    assert!(game.observe(O).library_selection.is_none());
    reject(&mut game, Action::FinishLibrarySelection);
    act(&mut game, Action::SelectLibrary { card: shock });
    act(&mut game, Action::SelectLibrary { card: mountain });
    act(&mut game, Action::FinishLibrarySelection);
    game.move_object(source, Zone::Graveyard);
    assert!(
        !game
            .observe(O)
            .objects
            .iter()
            .any(|object| object.id == shock)
    );
    act(&mut game, Action::PlayLand { card: mountain });
    assert_eq!(game.object_view(mountain).controller, P);
    game.state.players[P.index()].mana = [0, 0, 0, 0, 0, 1];
    act(&mut game, Action::AnnounceExile { card: shock });
    target(&mut game, Target::Player(O));
    finish_targets(&mut game);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[P.index()].mana, [0; 6]);
    assert_eq!(game.state.players[O.index()].life, 18);
    assert_eq!(game.object_view(shock).owner, O);
}

#[test]
fn carnage_pays_mayhem_and_the_returned_creature_sacrifices_itself_after_damage() {
    let mut game = scenario(P);
    let carnage = put(&mut game, P, "Carnage, Crimson Chaos", Zone::Hand);
    let hawk = put(&mut game, P, "Healer's Hawk", Zone::Graveyard);
    let large = put(&mut game, P, "Hill Giant", Zone::Graveyard);
    game.discard_batch(&[carnage], None);
    game.state.players[P.index()].mana = [0, 0, 1, 1, 0, 0];
    act(&mut game, Action::AnnounceMayhem { card: carnage });
    pay(&mut game);
    pass_pair(&mut game);
    reject(&mut game, target_action(Target::Card(large)));
    target(&mut game, Target::Card(hawk));
    finish_targets(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[P.index()].mana, [0; 6]);
    assert!(
        game.object_view(carnage)
            .keywords
            .contains(Keywords::TRAMPLE)
    );
    game.state.objects[hawk as usize].summoning_sick = false;
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    reject(&mut game, Action::FinishAttackers);
    start_attack(&mut game, &[hawk]);
    unblocked_damage(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[O.index()].life, 19);
    assert_eq!(game.object_view(hawk).zone, Zone::Graveyard);
}

#[test]
fn chameleon_pays_mayhem_and_copies_only_an_own_creature_keeping_his_name() {
    let mut game = scenario(P);
    let own = put(&mut game, P, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, O, "Hill Giant", Zone::Battlefield);
    let chameleon = put(&mut game, P, "Chameleon, Master of Disguise", Zone::Hand);
    game.discard_batch(&[chameleon], None);
    game.state.players[P.index()].mana = [0, 1, 0, 0, 0, 2];
    act(&mut game, Action::AnnounceMayhem { card: chameleon });
    pay(&mut game);
    pass_pair(&mut game);
    let enemy = game.object_ref(enemy);
    reject(
        &mut game,
        Action::ChooseEntryCopy {
            object: Some(enemy),
        },
    );
    let choice = Some(game.object_ref(own));
    act(&mut game, Action::ChooseEntryCopy { object: choice });
    settle(&mut game);
    let copy = game.object_view(chameleon);
    assert_eq!(copy.name, "Chameleon, Master of Disguise");
    assert!(copy.has_subtype("Bear"));
    assert!(!copy.is_legendary());
    assert_eq!(stats(&game, chameleon), (2, 2));
    assert_eq!(game.state.players[P.index()].mana, [0; 6]);
}

#[test]
fn cheering_crowd_can_be_cast_with_red_and_pays_the_active_players_colorless_mana() {
    let mut game = scenario(P);
    let crowd = put(&mut game, P, "Cheering Crowd", Zone::Hand);
    game.state.players[P.index()].mana = [0, 0, 0, 1, 0, 1];
    act(&mut game, Action::AnnounceCast { card: crowd });
    choose_spell_cost_if_needed(&mut game);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[P.index()].mana, [0; 6]);
    assert_eq!(stats(&game, crowd), (2, 2));
    game.add_counter(crowd, CounterKind::Charge, 2);
    game.state.active = O;
    game.state.priority = O;
    game.state.players[O.index()].mana = [0; 6];
    game.state.step = Step::Draw;
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(O));
    let saved = game.snapshot();
    act(&mut game, Action::ChooseOptional { accept: false });
    assert_eq!(plus_one(&game, crowd), 0);
    assert_eq!(game.state.players[O.index()].mana, [0; 6]);
    game.restore(&saved).unwrap();
    act(&mut game, Action::ChooseOptional { accept: true });
    settle(&mut game);
    assert_eq!(plus_one(&game, crowd), 1);
    assert_eq!(game.state.players[O.index()].mana, [0, 0, 0, 0, 0, 3]);
    assert_eq!(game.state.players[P.index()].mana, [0; 6]);
}

fn mana_ability(game: &mut Game, source: CardId, ability: u16, wanted: Mana) {
    let direct = game.legal_actions().into_iter().find(|action| {
        matches!(action, Action::ActivateMana { card, ability: slot, production, .. }
            if *card == source && *slot == ability && *production == wanted)
    });
    if let Some(direct) = direct {
        act(game, direct);
    } else {
        choose(game, |action| {
            matches!(action, Action::AnnounceMana { card, ability: slot, production, .. }
                if *card == source && *slot == ability && [wanted, [0; 6]].contains(production))
        });
        if matches!(game.state.pending, Pending::PayCosts { .. }) {
            pay(game);
        }
    }
    if matches!(game.state.pending, Pending::EffectMana { .. }) {
        act(game, Action::ChooseMana { production: wanted });
    }
}

#[test]
fn city_pigeon_leaves_and_common_crook_dies_under_their_last_controllers() {
    for (name, subtype, dies_only) in [
        ("City Pigeon", "Food", false),
        ("Common Crook", "Treasure", true),
    ] {
        let mut game = scenario(P);
        let card = put(&mut game, P, name, Zone::Hand);
        cast(&mut game, card, None);
        settle(&mut game);
        assert_eq!(
            game.object_view(card).keywords.contains(Keywords::FLYING),
            !dies_only
        );
        let object = game.object_ref(card);
        game.change_controller(object, O);
        game.move_object(card, Zone::Hand);
        game.prepare_triggers();
        settle(&mut game);
        if dies_only {
            assert!(
                !game
                    .observe(O)
                    .objects
                    .iter()
                    .any(|object| object.is_token() && object.has_subtype(subtype))
            );
            game.move_object(card, Zone::Battlefield);
            let object = game.object_ref(card);
            game.change_controller(object, O);
            game.move_object(card, Zone::Graveyard);
            game.prepare_triggers();
            settle(&mut game);
        }
        let treasure_or_food = token(&game, O, subtype);
        game.state.priority = O;
        if dies_only {
            game.state.players[O.index()].mana = [0; 6];
            mana_ability(&mut game, treasure_or_food, 0, BLUE);
            assert_eq!(game.state.players[O.index()].mana, BLUE);
        } else {
            game.state.players[O.index()].mana = [0, 0, 0, 0, 0, 2];
            activate(&mut game, treasure_or_food, 0, None);
            settle(&mut game);
            assert_eq!(game.state.players[O.index()].life, 23);
        }
        assert_ne!(game.object_view(treasure_or_food).zone, Zone::Battlefield);
    }
}

#[test]
fn cosmic_spider_man_grants_powers_to_own_spiders_including_a_kindred_permanent() {
    let powers = Keywords::FLYING
        .union(Keywords::FIRST_STRIKE)
        .union(Keywords::TRAMPLE)
        .union(Keywords::LIFELINK)
        .union(Keywords::HASTE);
    let mut game = scenario(P);
    let own = put(&mut game, P, "Treetop Snarespinner", Zone::Battlefield);
    let enemy = put(&mut game, O, "Treetop Snarespinner", Zone::Battlefield);
    let kindred = put(&mut game, P, "Boggart Mischief", Zone::Battlefield);
    let mut recipe = CopyRecipe::new(game.object_view(kindred).card);
    recipe.replacement_subtype = Some(CreatureType::from_name("Spider").unwrap());
    game.set_copy_recipe(game.object_ref(kindred), recipe);
    let source = put(&mut game, P, "Cosmic Spider-Man", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(game.object_view(source).keywords.contains(powers));
    assert!(!game.is_creature(kindred));
    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    settle(&mut game);
    for spider in [own, kindred] {
        assert!(game.object_view(spider).keywords.contains(powers));
    }
    assert!(!game.object_view(enemy).keywords.intersects(powers));
    let late = put(&mut game, P, "Treetop Snarespinner", Zone::Battlefield);
    assert!(!game.object_view(late).keywords.intersects(powers));
}

#[test]
fn costume_closet_moves_one_counter_and_replenishes_from_modified_departures() {
    let mut game = scenario(P);
    let closet = put(&mut game, P, "Costume Closet", Zone::Hand);
    let creature = put(&mut game, P, "Bear Cub", Zone::Battlefield);
    let ordinary = put(&mut game, P, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, O, "Bear Cub", Zone::Battlefield);
    cast(&mut game, closet, None);
    settle(&mut game);
    assert_eq!(plus_one(&game, closet), 2);
    let action = announce_activation(closet, 0);
    act(&mut game, action.clone());
    reject(&mut game, target_action(Target::Card(enemy)));
    target(&mut game, Target::Card(creature));
    finish_targets(&mut game);
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    settle(&mut game);
    assert_eq!(plus_one(&game, closet), 1);
    assert_eq!(plus_one(&game, creature), 1);
    assert!(game.object_view(closet).tapped);

    game.move_object(ordinary, Zone::Exile);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(plus_one(&game, closet), 1);
    game.move_object(creature, Zone::Exile);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(plus_one(&game, closet), 2);
}

#[test]
fn daily_bugle_building_has_separate_colorless_filter_and_legendary_menace_costs() {
    let mut game = scenario(P);
    let land = put(&mut game, P, "Daily Bugle Building", Zone::Hand);
    let legend = put(&mut game, O, "Aunt May", Zone::Battlefield);
    let ordinary = put(&mut game, O, "Bear Cub", Zone::Battlefield);
    act(&mut game, Action::PlayLand { card: land });
    assert!(!game.object_view(land).tapped);
    game.state.players[P.index()].mana = [0; 6];
    assert!(!game.legal_actions().iter().any(|action| matches!(
        action, Action::ActivateMana { card, ability: 1, .. }
            | Action::AnnounceMana { card, ability: 1, .. } if *card == land)));
    mana_ability(&mut game, land, 0, [0, 0, 0, 0, 0, 1]);
    assert_eq!(game.state.players[P.index()].mana, [0, 0, 0, 0, 0, 1]);
    game.state.objects[land as usize].tapped = false;
    mana_ability(&mut game, land, 1, BLUE);
    assert_eq!(game.state.players[P.index()].mana, BLUE);
    game.state.objects[land as usize].tapped = false;
    act(&mut game, announce_activation(land, 2));
    reject(&mut game, target_action(Target::Card(ordinary)));
    target(&mut game, Target::Card(legend));
    finish_targets(&mut game);
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(legend).keywords.contains(Keywords::MENACE));
    assert!(game.object_view(land).tapped);
    assert_eq!(game.state.players[P.index()].mana, [0; 6]);
}

#[test]
fn bugle_reporters_and_damage_control_resolve_each_printed_mode() {
    for (name, selected) in [
        ("Daily Bugle Reporters", 0),
        ("Daily Bugle Reporters", 1),
        ("Damage Control Crew", 0),
        ("Damage Control Crew", 1),
    ] {
        let mut game = scenario(P);
        let own_small = put(&mut game, P, "Bear Cub", Zone::Graveyard);
        let own_large = put(&mut game, P, "Hedron Archive", Zone::Graveyard);
        let artifact = put(&mut game, O, "Swiftfoot Boots", Zone::Battlefield);
        let creature = put(&mut game, O, "Bear Cub", Zone::Battlefield);
        let source = put(&mut game, P, name, Zone::Hand);
        cast(&mut game, source, None);
        pass_pair(&mut game);
        mode(&mut game, selected);
        match (name, selected) {
            ("Daily Bugle Reporters", 0) => {
                target(&mut game, Target::Card(creature));
                target(&mut game, Target::Card(source));
                finish_targets(&mut game);
                game.move_object(creature, Zone::Exile);
                settle(&mut game);
                assert_eq!(plus_one(&game, source), 1);
            }
            _ => {
                let (chosen, wrong, destination) = match (name, selected) {
                    ("Daily Bugle Reporters", 1) => (own_small, own_large, Zone::Hand),
                    ("Damage Control Crew", 0) => (own_large, own_small, Zone::Hand),
                    ("Damage Control Crew", 1) => (artifact, creature, Zone::Exile),
                    _ => unreachable!(),
                };
                reject(&mut game, target_action(Target::Card(wrong)));
                target(&mut game, Target::Card(chosen));
                finish_targets(&mut game);
                settle(&mut game);
                assert_eq!(game.object_view(chosen).zone, destination);
            }
        }
    }
}

#[test]
fn henchmen_flash_and_connive_on_attack_with_only_nonland_discards_growing_them() {
    for nonland in [false, true] {
        let mut game = scenario(O);
        let henchmen = put(&mut game, P, "Doc Ock's Henchmen", Zone::Hand);
        let discarded = put(
            &mut game,
            P,
            if nonland { "Shock" } else { "Forest" },
            Zone::Hand,
        );
        game.state.priority = P;
        cast(&mut game, henchmen, None);
        settle(&mut game);
        game.state.active = P;
        game.state.priority = P;
        game.state.objects[henchmen as usize].summoning_sick = false;
        start_attack(&mut game, &[henchmen]);
        pass_pair(&mut game);
        act(&mut game, Action::Discard { card: discarded });
        settle(&mut game);
        assert_eq!(game.hand_count(P), 1);
        assert_eq!(plus_one(&game, henchmen), u64::from(nonland));
    }
}

#[test]
fn tentacles_attach_untargeted_to_a_large_entrant_and_can_be_equipped_later() {
    let mut game = scenario(P);
    let tentacles = put(&mut game, P, "Doc Ock's Tentacles", Zone::Hand);
    cast(&mut game, tentacles, None);
    settle(&mut game);
    let small = put(&mut game, P, "Hill Giant", Zone::Hand);
    cast(&mut game, small, None);
    settle(&mut game);
    assert_eq!(game.object_view(tentacles).attached_to, None);
    let large = put(&mut game, P, "Shivan Dragon", Zone::Hand);
    cast(&mut game, large, None);
    pass_pair(&mut game);
    assert!(game.state.stack.last().unwrap().targets.is_empty());
    pass_pair(&mut game);
    let before = game.snapshot();
    act(&mut game, Action::ChooseOptional { accept: false });
    assert_eq!(game.object_view(tentacles).attached_to, None);
    game.restore(&before).unwrap();
    act(&mut game, Action::ChooseOptional { accept: true });
    settle(&mut game);
    assert_eq!(
        game.object_view(tentacles).attached_to,
        Some(game.object_ref(large))
    );
    assert_eq!(stats(&game, large), (9, 9));
    game.state.players[P.index()].mana = [0, 0, 0, 0, 0, 5];
    activate(&mut game, tentacles, 0, Some(Target::Card(small)));
    settle(&mut game);
    assert_eq!(stats(&game, small), (7, 7));
    assert_eq!(game.state.players[P.index()].mana, [0; 6]);
}

#[test]
fn sinister_scientist_uses_own_graveyard_and_another_controlled_villain() {
    let mut game = scenario(P);
    let scientist = put(&mut game, P, "Doc Ock, Sinister Scientist", Zone::Hand);
    for _ in 0..7 {
        put(&mut game, P, "Forest", Zone::Graveyard);
    }
    put(&mut game, O, "Forest", Zone::Graveyard);
    put(&mut game, O, "Common Crook", Zone::Battlefield);
    cast(&mut game, scientist, None);
    settle(&mut game);
    assert_eq!(stats(&game, scientist), (4, 5));
    assert!(
        !game
            .object_view(scientist)
            .keywords
            .contains(Keywords::HEXPROOF)
    );
    put(&mut game, P, "Forest", Zone::Graveyard);
    let villain = put(&mut game, P, "Common Crook", Zone::Battlefield);
    assert_eq!(stats(&game, scientist), (8, 8));
    assert!(
        game.object_view(scientist)
            .keywords
            .contains(Keywords::HEXPROOF)
    );
    game.move_object(villain, Zone::Exile);
    assert!(
        !game
            .object_view(scientist)
            .keywords
            .contains(Keywords::HEXPROOF)
    );
}

#[test]
fn doctor_octopus_buffs_other_villains_and_refills_to_eight_on_his_own_end_step() {
    let mut game = scenario(P);
    let source = put(&mut game, P, "Doctor Octopus, Master Planner", Zone::Hand);
    let villain = put(&mut game, P, "Common Crook", Zone::Battlefield);
    let opponent = put(&mut game, O, "Common Crook", Zone::Battlefield);
    cast(&mut game, source, None);
    settle(&mut game);
    assert_eq!(stats(&game, source), (4, 8));
    assert_eq!(stats(&game, villain), (4, 4));
    assert_eq!(stats(&game, opponent), (2, 2));
    assert_eq!(game.maximum_hand_size(P), Some(8));
    for _ in 0..5 {
        put(&mut game, P, "Forest", Zone::Hand);
    }
    game.begin_step(Step::End);
    game.prepare_triggers();
    put(&mut game, P, "Forest", Zone::Hand);
    settle(&mut game);
    assert_eq!(game.hand_count(P), 8);
    assert_eq!(game.state.players[P.index()].cards_drawn_this_turn, 2);
    game.begin_step(Step::End);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
}

#[test]
fn eddie_reanimates_and_transforms_in_place_into_a_paid_sacrifice_and_deploy_trigger() {
    let mut game = scenario(P);
    let eddie = put(&mut game, P, "Eddie Brock", Zone::Hand);
    let hawk = put(&mut game, P, "Healer's Hawk", Zone::Graveyard);
    let large = put(&mut game, P, "Bear Cub", Zone::Graveyard);
    let donor = put(&mut game, P, "Grizzly Bears", Zone::Battlefield);
    let deploy = put(&mut game, P, "Swiftfoot Boots", Zone::Hand);
    let too_large = put(&mut game, P, "Hill Giant", Zone::Hand);
    cast(&mut game, eddie, None);
    pass_pair(&mut game);
    reject(&mut game, target_action(Target::Card(large)));
    target(&mut game, Target::Card(hawk));
    finish_targets(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(hawk).zone, Zone::Battlefield);

    let reference = game.object_ref(eddie);
    game.add_counter(eddie, CounterKind::Charge, 1);
    game.state.players[P.index()].mana = [0, 0, 1, 1, 1, 3];
    activate(&mut game, eddie, 0, None);
    settle(&mut game);
    assert_eq!(game.state.players[P.index()].mana, [0; 6]);
    assert_eq!(game.object_ref(eddie), reference);
    let venom = game.object_view(eddie);
    assert_eq!(venom.name, "Venom, Lethal Protector");
    assert!(
        venom.keywords.contains(
            Keywords::MENACE
                .union(Keywords::HASTE)
                .union(Keywords::TRAMPLE)
        )
    );
    assert_eq!(stats(&game, eddie), (5, 5));
    assert_eq!(game.counter_count(eddie, CounterKind::Charge), 1);
    start_attack(&mut game, &[eddie]);
    pass_pair(&mut game);
    reject(&mut game, Action::Sacrifice { object: reference });
    let object = game.object_ref(donor);
    act(&mut game, Action::Sacrifice { object });
    assert_eq!(game.state.players[P.index()].cards_drawn_this_turn, 2);
    assert!(game.observe(O).effect_choice.is_none());
    let too_large = game.object_ref(too_large);
    reject(&mut game, Action::PutHandPermanent { object: too_large });
    let object = game.object_ref(deploy);
    act(&mut game, Action::PutHandPermanent { object });
    settle(&mut game);
    assert_eq!(game.object_view(donor).zone, Zone::Graveyard);
    assert_eq!(game.object_view(deploy).zone, Zone::Battlefield);
}

#[test]
fn venom_can_also_be_cast_as_the_back_without_eddies_enters_trigger() {
    let mut game = scenario(P);
    let front = put(&mut game, P, "Eddie Brock", Zone::Hand);
    let hawk = put(&mut game, P, "Healer's Hawk", Zone::Graveyard);
    game.state.players[P.index()].mana = [0, 0, 1, 1, 1, 3];
    act(&mut game, Action::AnnounceBack { card: front });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(front).name, "Venom, Lethal Protector");
    assert_eq!(game.object_view(hawk).zone, Zone::Graveyard);
    assert_eq!(game.state.players[P.index()].mana, [0; 6]);
}

#[test]
fn eerie_gravestone_draws_and_recovers_only_a_creature_from_the_actual_mill() {
    for replace in [false, true] {
        let mut game = scenario(P);
        let grave = put(&mut game, P, "Eerie Gravestone", Zone::Hand);
        let unrelated = put(&mut game, P, "Bear Cub", Zone::Graveyard);
        cast(&mut game, grave, None);
        settle(&mut game);
        assert_eq!(game.hand_count(P), 1);
        if replace {
            put(&mut game, O, "Rest in Peace", Zone::Battlefield);
        }
        let milled = put(&mut game, P, "Grizzly Bears", Zone::Library);
        let old = game.object_ref(unrelated);
        game.state.players[P.index()].mana = [0, 0, 1, 0, 0, 1];
        activate(&mut game, grave, 0, None);
        pass_pair(&mut game);
        assert_eq!(
            game.object_view(milled).zone,
            if replace {
                Zone::Exile
            } else {
                Zone::Graveyard
            }
        );
        reject(&mut game, Action::SelectMilled { object: old });
        let object = game.object_ref(milled);
        act(&mut game, Action::SelectMilled { object });
        if game
            .legal_actions()
            .contains(&Action::FinishMilledSelection)
        {
            act(&mut game, Action::FinishMilledSelection);
        }
        settle(&mut game);
        assert_eq!(game.object_view(milled).zone, Zone::Hand);
        assert_eq!(
            game.object_view(grave).zone,
            if replace {
                Zone::Exile
            } else {
                Zone::Graveyard
            }
        );
        assert_eq!(game.state.players[P.index()].mana, [0; 6]);
    }
}

#[test]
fn electros_bolt_pays_mayhem_only_for_the_exact_discard_and_deals_four() {
    let mut game = scenario(P);
    let bolt = put(&mut game, P, "Electro's Bolt", Zone::Hand);
    let unrelated = put(&mut game, P, "Electro's Bolt", Zone::Graveyard);
    let victim = put(&mut game, O, "Shivan Dragon", Zone::Battlefield);
    game.discard_batch(&[bolt], None);
    game.state.players[P.index()].mana = [0, 0, 0, 1, 0, 1];
    reject(&mut game, Action::AnnounceMayhem { card: unrelated });
    act(&mut game, Action::AnnounceMayhem { card: bolt });
    target(&mut game, Target::Card(victim));
    finish_targets(&mut game);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(victim).damage, 4);
    assert_eq!(game.object_view(bolt).zone, Zone::Graveyard);
    assert_eq!(game.state.players[P.index()].mana, [0; 6]);
}

#[test]
fn electro_retains_red_from_casts_and_pays_for_a_separate_leave_damage_trigger() {
    let mut game = scenario(P);
    let electro = put(&mut game, P, "Electro, Assaulting Battery", Zone::Hand);
    cast(&mut game, electro, None);
    settle(&mut game);
    assert!(
        game.object_view(electro)
            .keywords
            .contains(Keywords::FLYING)
    );
    let shock = put(&mut game, P, "Shock", Zone::Hand);
    game.state.players[P.index()].mana = [0, 1, 0, 1, 0, 0];
    cast(&mut game, shock, Some(Target::Player(O)));
    pass_pair(&mut game);
    assert_eq!(game.state.players[P.index()].mana, [0, 1, 0, 1, 0, 0]);
    settle(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.state.players[P.index()].mana, [0, 0, 0, 1, 0, 0]);

    game.move_object(electro, Zone::Exile);
    game.prepare_triggers();
    pass_pair(&mut game);
    act(&mut game, Action::ChooseOptional { accept: true });
    act(&mut game, Action::ChooseX { value: 1 });
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    target(&mut game, Target::Player(O));
    finish_targets(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[O.index()].life, 17);
    assert_eq!(game.state.players[P.index()].mana, [0; 6]);
}

#[test]
fn ezekiel_targets_only_a_spider_he_controls_at_the_beginning_of_his_combat() {
    let mut game = scenario(P);
    let source = put(&mut game, P, "Ezekiel Sims, Spider-Totem", Zone::Hand);
    let ordinary = put(&mut game, P, "Bear Cub", Zone::Battlefield);
    let enemy = put(&mut game, O, "Treetop Snarespinner", Zone::Battlefield);
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(game.object_view(source).keywords.contains(Keywords::REACH));
    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    reject(&mut game, target_action(Target::Card(ordinary)));
    reject(&mut game, target_action(Target::Card(enemy)));
    target(&mut game, Target::Card(source));
    finish_targets(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, source), (5, 7));
}

#[test]
fn flash_thompson_can_tap_and_untap_the_same_target_off_turn() {
    let mut game = scenario(O);
    let thompson = put(&mut game, P, "Flash Thompson, Spider-Fan", Zone::Hand);
    let target_card = put(&mut game, O, "Bear Cub", Zone::Battlefield);
    game.state.priority = P;
    cast(&mut game, thompson, None);
    pass_pair(&mut game);
    mode(&mut game, 2);
    for _ in 0..2 {
        target(&mut game, Target::Card(target_card));
        finish_targets(&mut game);
    }
    settle(&mut game);
    assert!(!game.object_view(target_card).tapped);
    assert_eq!(game.object_view(thompson).zone, Zone::Battlefield);
}

#[test]
fn flying_octobot_grows_once_for_another_villain_and_resets_on_the_next_turn() {
    let mut game = scenario(P);
    let octobot = put(&mut game, P, "Flying Octobot", Zone::Hand);
    cast(&mut game, octobot, None);
    settle(&mut game);
    assert_eq!(plus_one(&game, octobot), 0);
    assert!(
        game.object_view(octobot)
            .keywords
            .contains(Keywords::FLYING)
    );
    for _ in 0..2 {
        let villain = put(&mut game, P, "Common Crook", Zone::Hand);
        cast(&mut game, villain, None);
        settle(&mut game);
        assert_eq!(plus_one(&game, octobot), 1);
    }
    game.state.step = Step::End;
    pass_pair(&mut game);
    assert_eq!(game.state.active, O);
    let villain = put(&mut game, P, "Common Crook", Zone::Hand);
    game.move_object(villain, Zone::Battlefield);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(plus_one(&game, octobot), 2);
}

#[test]
fn forest_and_gallant_citizen_supply_and_consume_the_printed_colors() {
    for payment in [[0, 0, 0, 0, 2, 0], [1, 0, 0, 0, 1, 0], [2, 0, 0, 0, 0, 0]] {
        let mut game = scenario(P);
        game.state.players[P.index()].mana = [0; 6];
        let forest = put(&mut game, P, "Forest", Zone::Hand);
        act(&mut game, Action::PlayLand { card: forest });
        act(&mut game, Action::TapMana { card: forest });
        assert_eq!(game.state.players[P.index()].mana, [0, 0, 0, 0, 1, 0]);
        assert!(game.object_view(forest).has_subtype("Forest"));
        let citizen = put(&mut game, P, "Gallant Citizen", Zone::Hand);
        game.state.players[P.index()].mana = payment;
        act(&mut game, Action::AnnounceCast { card: citizen });
        choose_spell_cost_if_needed(&mut game);
        pay(&mut game);
        settle(&mut game);
        assert_eq!(game.state.players[P.index()].mana, [0; 6]);
        assert_eq!(stats(&game, citizen), (1, 1));
        assert_eq!(game.hand_count(P), 1);
    }
}

#[test]
fn friendly_neighborhood_creates_three_citizens_and_grants_the_lands_controller_the_activation() {
    let mut game = scenario(P);
    let land = put(&mut game, O, "Forest", Zone::Battlefield);
    let creature = put(&mut game, O, "Bear Cub", Zone::Battlefield);
    let aura = put(&mut game, P, "Friendly Neighborhood", Zone::Hand);
    cast(&mut game, aura, Some(Target::Card(land)));
    settle(&mut game);
    assert_eq!(
        game.object_view(aura).attached_to,
        Some(game.object_ref(land))
    );
    let citizens: Vec<_> = game
        .observe(P)
        .objects
        .into_iter()
        .filter(|object| {
            object.zone == Zone::Battlefield && object.controller == P && object.is_token()
        })
        .collect();
    assert_eq!(citizens.len(), 3);
    for citizen in citizens {
        assert!(citizen.has_subtype("Human") && citizen.has_subtype("Citizen"));
        assert_eq!(citizen.colors, 17);
        assert_eq!(stats(&game, citizen.id), (1, 1));
    }
    let action = Action::AnnounceAttachedAbility {
        grantor: game.object_ref(aura),
        ability: 0,
    };
    reject(&mut game, action.clone());
    game.state.active = O;
    game.state.priority = O;
    game.state.players[O.index()].mana = [0, 0, 0, 0, 0, 1];
    act(&mut game, action);
    target(&mut game, Target::Card(creature));
    finish_targets(&mut game);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, creature), (3, 3));
    assert!(game.object_view(land).tapped);
    assert!(!game.object_view(aura).tapped);
}

#[test]
fn green_goblin_discards_on_attack_and_draws_for_all_earlier_and_current_discards() {
    let mut game = scenario(P);
    let source = put(&mut game, P, "Green Goblin, Revenant", Zone::Hand);
    let previous = put(&mut game, P, "Forest", Zone::Hand);
    let current = put(&mut game, P, "Mountain", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(
        game.object_view(source)
            .keywords
            .contains(Keywords::FLYING.union(Keywords::DEATHTOUCH))
    );
    game.discard_batch(&[previous], None);
    game.state.objects[source as usize].summoning_sick = false;
    start_attack(&mut game, &[source]);
    pass_pair(&mut game);
    act(&mut game, Action::Discard { card: current });
    settle(&mut game);
    assert_eq!(game.object_view(current).zone, Zone::Graveyard);
    assert_eq!(game.hand_count(P), 2);
}

#[test]
fn grow_extra_arms_costs_one_less_only_when_it_targets_a_spider() {
    for spider_target in [false, true] {
        let mut game = scenario(P);
        let spider = put(&mut game, P, "Treetop Snarespinner", Zone::Battlefield);
        let other = put(&mut game, P, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut game, P, "Grow Extra Arms", Zone::Hand);
        game.state.players[P.index()].mana = [0, 0, 0, 0, 1, u32::from(!spider_target)];
        act(&mut game, Action::AnnounceCast { card: spell });
        if spider_target {
            reject(&mut game, target_action(Target::Card(other)));
        }
        let recipient = if spider_target { spider } else { other };
        let (power, toughness) = stats(&game, recipient);
        target(&mut game, Target::Card(recipient));
        finish_targets(&mut game);
        pay(&mut game);
        settle(&mut game);
        assert_eq!(stats(&game, recipient), (power + 4, toughness + 4));
        assert_eq!(game.state.players[P.index()].mana, [0; 6]);
    }
}

#[test]
fn guy_in_the_chair_taps_for_any_color_and_supports_a_spider_at_sorcery_speed() {
    let mut game = scenario(P);
    let source = put(&mut game, P, "Guy in the Chair", Zone::Hand);
    let spider = put(&mut game, O, "Treetop Snarespinner", Zone::Battlefield);
    let ordinary = put(&mut game, P, "Bear Cub", Zone::Battlefield);
    cast(&mut game, source, None);
    settle(&mut game);
    reject(&mut game, announce_activation(source, 1));
    game.state.objects[source as usize].summoning_sick = false;
    game.state.players[P.index()].mana = [0; 6];
    mana_ability(&mut game, source, 0, WHITE);
    assert_eq!(game.state.players[P.index()].mana, WHITE);
    assert!(game.object_view(source).tapped);
    game.state.objects[source as usize].tapped = false;
    game.state.players[P.index()].mana = [0, 0, 0, 0, 1, 2];
    act(&mut game, announce_activation(source, 1));
    reject(&mut game, target_action(Target::Card(ordinary)));
    target(&mut game, Target::Card(spider));
    finish_targets(&mut game);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(plus_one(&game, spider), 1);
    assert!(game.object_view(source).tapped);
    assert_eq!(game.state.players[P.index()].mana, [0; 6]);
}

#[test]
fn gwen_stacys_exile_permission_survives_transform_and_ghost_spider_spends_mixed_counters() {
    let mut game = scenario(P);
    let gwen = put(&mut game, P, "Gwen Stacy", Zone::Hand);
    let land = put(&mut game, P, "Forest", Zone::Library);
    cast(&mut game, gwen, None);
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    let reference = game.object_ref(gwen);
    game.state.players[P.index()].mana = [1, 1, 0, 1, 0, 2];
    activate(&mut game, gwen, 0, None);
    settle(&mut game);
    assert_eq!(game.object_ref(gwen), reference);
    assert_eq!(game.object_view(gwen).name, "Ghost-Spider");
    assert!(
        game.object_view(gwen).keywords.contains(
            Keywords::FLYING
                .union(Keywords::VIGILANCE)
                .union(Keywords::HASTE)
        )
    );
    assert_eq!(game.state.players[P.index()].mana, [0; 6]);
    act(&mut game, Action::PlayLand { card: land });
    settle(&mut game);
    assert_eq!(plus_one(&game, gwen), 1);

    let shock = put(&mut game, P, "Shock", Zone::Library);
    game.add_counter(gwen, CounterKind::Charge, 1);
    let mut counters = Counters::default();
    counters.add(CounterKind::PlusOne, 1);
    counters.add(CounterKind::Charge, 1);
    act(
        &mut game,
        Action::AnnounceActivate {
            card: gwen,
            ability: 0,
            counter_payment: counters,
        },
    );
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    settle(&mut game);
    assert_eq!(plus_one(&game, gwen), 0);
    assert_eq!(game.counter_count(gwen, CounterKind::Charge), 0);
    assert_eq!(game.object_view(shock).zone, Zone::Exile);

    game.state.players[P.index()].mana = [0, 0, 0, 1, 0, 0];
    act(&mut game, Action::AnnounceExile { card: shock });
    target(&mut game, Target::Player(O));
    finish_targets(&mut game);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(plus_one(&game, gwen), 1);
    assert_eq!(game.state.players[O.index()].life, 18);
}

#[test]
fn ghost_spider_cast_back_does_not_trigger_for_a_hand_land_or_exile_the_top_card() {
    let mut game = scenario(P);
    let source = put(&mut game, P, "Gwen Stacy", Zone::Hand);
    let hand_land = put(&mut game, P, "Forest", Zone::Hand);
    let top = game.top_library_card(P).unwrap();
    game.state.players[P.index()].mana = [1, 1, 0, 1, 0, 2];
    act(&mut game, Action::AnnounceBack { card: source });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(source).name, "Ghost-Spider");
    assert_eq!(game.object_view(top).zone, Zone::Library);
    act(&mut game, Action::PlayLand { card: hand_land });
    settle(&mut game);
    assert_eq!(plus_one(&game, source), 0);
}

#[test]
fn gwenom_grants_private_live_library_plays_and_the_spell_uses_life() {
    let mut game = scenario(P);
    let source = put(&mut game, P, "Gwenom, Remorseless", Zone::Hand);
    let shock = put(&mut game, P, "Shock", Zone::Library);
    cast(&mut game, source, None);
    settle(&mut game);
    assert!(
        game.object_view(source)
            .keywords
            .contains(Keywords::LIFELINK.union(Keywords::DEATHTOUCH))
    );
    let announce = Action::AnnounceLibraryLife {
        card: shock,
        face: CastFace::Normal,
        flash_surcharge: false,
    };
    reject(&mut game, announce.clone());
    game.state.objects[source as usize].summoning_sick = false;
    start_attack(&mut game, &[source]);
    pass_pair(&mut game);
    let visible_top =
        |viewer| game.observe(viewer).known_library_top[P.index()].map(|top| top.object);
    assert_eq!(visible_top(P), Some(game.object_ref(shock)));
    assert_eq!(visible_top(O), None);
    game.move_object(source, Zone::Exile);
    game.state.players[P.index()].mana = [0; 6];
    act(&mut game, announce);
    target(&mut game, Target::Player(O));
    finish_targets(&mut game);
    for _ in 0..8 {
        match game.state.pending {
            Pending::CastCosts { .. } => choose(&mut game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            }),
            Pending::PayCosts { .. } => pay(&mut game),
            Pending::Priority => break,
            ref pending => panic!("unexpected life cast decision {pending:?}"),
        }
    }
    settle(&mut game);
    assert_eq!(game.state.players[P.index()].life, 19);
    assert_eq!(game.state.players[O.index()].life, 18);
}

#[test]
fn heroes_hangout_exiles_two_and_selects_one_or_buffs_two_creatures() {
    for selected in [0, 1] {
        let mut game = scenario(P);
        let heroes = put(&mut game, P, "Heroes' Hangout", Zone::Hand);
        let first = put(&mut game, P, "Bear Cub", Zone::Battlefield);
        let second = put(&mut game, O, "Bear Cub", Zone::Battlefield);
        let unchosen = put(&mut game, P, "Island", Zone::Library);
        let chosen = put(&mut game, P, "Forest", Zone::Library);
        act(&mut game, Action::AnnounceCast { card: heroes });
        mode(&mut game, selected);
        if selected == 1 {
            target(&mut game, Target::Card(first));
            target(&mut game, Target::Card(second));
            finish_targets(&mut game);
        }
        pay(&mut game);
        if selected == 0 {
            pass_pair(&mut game);
            assert_eq!(game.object_view(unchosen).zone, Zone::Exile);
            assert_eq!(game.object_view(chosen).zone, Zone::Exile);
            let object = game.object_ref(chosen);
            act(&mut game, Action::SelectMilled { object });
            settle(&mut game);
            reject(&mut game, Action::PlayLand { card: unchosen });
            assert!(
                game.observe(P)
                    .exile_permissions
                    .iter()
                    .any(|permission| permission.object == object
                        && permission.expiry == ExilePlayExpiry::EndOfNextTurn)
            );
            act(&mut game, Action::PlayLand { card: chosen });
        } else {
            settle(&mut game);
            for id in [first, second] {
                assert_eq!(stats(&game, id), (3, 2));
                assert!(
                    game.object_view(id)
                        .keywords
                        .contains(Keywords::FIRST_STRIKE)
                );
            }
        }
    }
}

#[test]
fn hide_on_the_ceiling_exiles_exactly_x_and_returns_actual_cards_to_their_owners() {
    let mut game = scenario(P);
    let creature = put(&mut game, O, "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut game, P, "Swiftfoot Boots", Zone::Battlefield);
    let land = put(&mut game, P, "Forest", Zone::Battlefield);
    game.change_controller(game.object_ref(creature), P);
    let spell = put(&mut game, P, "Hide on the Ceiling", Zone::Hand);
    game.state.players[P.index()].mana = [0, 1, 0, 0, 0, 2];
    act(&mut game, Action::AnnounceCast { card: spell });
    act(&mut game, Action::ChooseX { value: 2 });
    target(&mut game, Target::Card(creature));
    reject(&mut game, Action::FinishTargets);
    reject(&mut game, target_action(Target::Card(land)));
    target(&mut game, Target::Card(artifact));
    finish_targets(&mut game);
    pay(&mut game);
    settle(&mut game);
    for id in [creature, artifact] {
        assert_eq!(game.object_view(id).zone, Zone::Exile);
    }
    assert_eq!(game.state.players[P.index()].mana, [0; 6]);
    game.begin_step(Step::End);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.object_view(creature).controller, O);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert_eq!(game.object_view(artifact).controller, P);
    assert_eq!(game.object_view(artifact).zone, Zone::Battlefield);
}

#[test]
fn hobgoblin_grows_once_per_discard_and_hot_dog_cart_creates_separate_food_and_mana() {
    let mut game = scenario(P);
    let hobgoblin = put(&mut game, P, "Hobgoblin, Mantled Marauder", Zone::Hand);
    let cart = put(&mut game, P, "Hot Dog Cart", Zone::Hand);
    cast(&mut game, hobgoblin, None);
    settle(&mut game);
    cast(&mut game, cart, None);
    settle(&mut game);
    assert!(
        game.object_view(hobgoblin)
            .keywords
            .contains(Keywords::FLYING.union(Keywords::HASTE))
    );
    let discarded = [
        put(&mut game, P, "Forest", Zone::Hand),
        put(&mut game, P, "Island", Zone::Hand),
    ];
    game.discard_batch(&discarded, None);
    game.prepare_triggers();
    order_triggers(&mut game);
    settle(&mut game);
    assert_eq!(stats(&game, hobgoblin), (5, 2));
    let food = token(&game, P, "Food");
    game.state.players[P.index()].mana = [0, 0, 0, 0, 0, 1];
    mana_ability(&mut game, cart, 0, BLUE);
    assert_eq!(game.state.players[P.index()].mana, [0, 1, 0, 0, 0, 1]);
    activate(&mut game, food, 0, None);
    settle(&mut game);
    assert_eq!(game.state.players[P.index()].life, 23);
    assert_eq!(game.object_view(cart).zone, Zone::Battlefield);
    assert_ne!(game.object_view(food).zone, Zone::Battlefield);
}

#[test]
fn hydro_man_grows_for_blue_and_untaps_as_a_mana_land_until_his_next_turn() {
    let mut game = scenario(P);
    let hydro = put(&mut game, P, "Hydro-Man, Fluid Felon", Zone::Hand);
    let blue = put(&mut game, P, "Divination", Zone::Hand);
    cast(&mut game, hydro, None);
    settle(&mut game);
    cast(&mut game, blue, None);
    settle(&mut game);
    assert_eq!(stats(&game, hydro), (3, 3));
    game.state.objects[hydro as usize].tapped = true;
    game.begin_step(Step::End);
    game.prepare_triggers();
    settle(&mut game);
    let land = game.object_view(hydro);
    assert_eq!(land.kinds, 2);
    assert!(!land.tapped);
    assert!(!land.has_subtype("Elemental"));
    game.state.players[P.index()].mana = [0; 6];
    choose(&mut game, |action| {
        matches!(action,
        Action::TapMana { card } | Action::TapGrantedMana { card, .. } if *card == hydro)
    });
    assert_eq!(game.state.players[P.index()].mana, BLUE);
    pass_pair(&mut game);
    assert_eq!(game.state.active, O);
    assert_eq!(game.object_view(hydro).kinds, 2);
    game.state.step = Step::End;
    pass_pair(&mut game);
    assert_eq!(game.state.active, P);
    assert_eq!(game.object_view(hydro).kinds, 1);
    assert!(game.object_view(hydro).has_subtype("Elemental"));
    assert_eq!(stats(&game, hydro), (2, 2));
}

#[test]
fn impostor_syndrome_copies_the_nontoken_dealer_using_last_known_copiable_values() {
    let mut game = scenario(P);
    let legend = put(&mut game, P, "Agent Venom", Zone::Battlefield);
    let citizen = spawn(
        &mut game,
        P,
        Card::from_catalogue_id(5314),
        Zone::Battlefield,
    );
    let enchantment = put(&mut game, P, "Impostor Syndrome", Zone::Hand);
    cast(&mut game, enchantment, None);
    settle(&mut game);
    game.add_counter(legend, CounterKind::PlusOne, 1);
    start_attack(&mut game, &[legend, citizen]);
    unblocked_damage(&mut game);
    assert_eq!(game.state.players[O.index()].life, 16);
    assert_eq!(game.state.stack.len(), 1);
    game.move_object(legend, Zone::Exile);
    settle(&mut game);
    let copy = token(&game, P, "Symbiote");
    let copied = game.object_view(copy);
    assert_eq!(copied.name, "Agent Venom");
    assert!(!copied.is_legendary());
    assert_eq!(plus_one(&game, copy), 0);
    assert_eq!(stats(&game, copy), (2, 3));
}

fn choose_spell_cost_if_needed(game: &mut Game) {
    let action = game
        .legal_actions()
        .into_iter()
        .find(|action| matches!(action, Action::ChooseSpellCost { .. }));
    if let Some(action) = action {
        act(game, action);
    }
}
