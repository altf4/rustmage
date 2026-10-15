use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;

fn named(game: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, p, Card::from_name(name).unwrap(), zone)
}
fn activate(game: &mut Game, id: CardId, target: Option<Target>) {
    let action = game.legal_actions().into_iter().find(|a|
        matches!(a, Action::Activate { card, target: chosen, .. } if *card == id && *chosen == target))
        .expect("legal activation");
    act(game, action);
}
fn has_activation(game: &Game, id: CardId) -> bool {
    game.legal_actions().iter().any(|a| matches!(a, Action::Activate { card, .. } | Action::BeginActivate { card, .. } if *card == id))
}

#[test]
fn haste_self_sacrifice_pays_before_priority_and_damage_survives_source_departure() {
    let mut game = scenario();
    let firebrand = named(&mut game, Player::P0, "Fanatical Firebrand", Zone::Hand);
    game.move_object(firebrand, Zone::Battlefield);
    assert!(game.object_view(firebrand).summoning_sick);
    let original = game.object_ref(firebrand);
    activate(&mut game, firebrand, Some(Target::Player(Player::P1)));
    assert_eq!(game.object_view(firebrand).zone, Zone::Graveyard);
    assert_eq!(game.state.stack.len(), 1);
    let ability = &game.state.stack[0];
    assert_eq!(ability.object, original);
    assert!(matches!(
        ability.kind,
        StackKind::ActivatedAbility { ability: 0 }
    ));
    assert!(!game.target_matches(
        TargetRule::Spell,
        Player::P1,
        original,
        Target::Spell(firebrand)
    ));
    let before = game.snapshot();
    // Returning the source cannot move or erase the captured stack ability.
    game.move_object(firebrand, Zone::Battlefield);
    pass_pair(&mut game);
    assert_eq!(game.state.players[1].life, 19);
    assert_eq!(game.object_view(firebrand).zone, Zone::Battlefield);
    let after = game.snapshot();
    game.restore(&before);
    game.move_object(firebrand, Zone::Battlefield);
    pass_pair(&mut game);
    assert_eq!(game.snapshot(), after);
}

#[test]
fn tap_cost_enforces_sickness_tapped_status_turn_and_exact_payment_atomically() {
    let mut game = scenario();
    let duelist = named(&mut game, Player::P0, "Deadeye Duelist", Zone::Hand);
    game.move_object(duelist, Zone::Battlefield);
    game.state.players[0].mana[0] = 1;
    assert!(!has_activation(&game, duelist));
    let forged = Action::Activate {
        card: duelist,
        ability: 0,
        target: Some(Target::Player(Player::P1)),
        payment: [1, 0, 0, 0, 0],
        counter_payment: Counters::default(),
    };
    let before = game.snapshot();
    let revision = game.revision;
    assert_eq!(
        game.apply(Player::P0, forged.clone()),
        Err(Error::IllegalAction)
    );
    assert_eq!(game.snapshot(), before);
    assert_eq!(game.revision, revision);
    game.begin_turn();
    assert!(has_activation(&game, duelist));
    let before = game.snapshot();
    assert_eq!(
        game.apply(
            Player::P0,
            Action::Activate {
                card: duelist,
                ability: 0,
                target: Some(Target::Player(Player::P1)),
                payment: [2, 0, 0, 0, 0],
                counter_payment: Counters::default(),
            }
        ),
        Err(Error::IllegalAction)
    );
    assert_eq!(game.snapshot(), before);
    act(&mut game, forged);
    assert!(game.object_view(duelist).tapped);
    assert_eq!(game.state.players[0].mana[0], 0);
    game.state.players[0].mana[0] = 1;
    assert!(!has_activation(&game, duelist));
    pass_pair(&mut game);
    assert_eq!(game.state.players[1].life, 19);
}

#[test]
fn sorcery_only_abilities_require_active_main_phase_and_empty_stack() {
    let mut game = scenario();
    let tenderfoot = named(
        &mut game,
        Player::P0,
        "Intrepid Tenderfoot",
        Zone::Battlefield,
    );
    game.state.players[0].mana[0] = 10;
    assert!(has_activation(&game, tenderfoot));
    game.state.step = Step::Upkeep;
    assert!(!has_activation(&game, tenderfoot));
    game.state.step = Step::Main1;
    let bolt = spawn(&mut game, Player::P0, Card::LightningBolt, Zone::Hand);
    game.state.players[0].mana[3] = 1;
    cast(&mut game, bolt, Some(Target::Player(Player::P1)));
    assert!(!has_activation(&game, tenderfoot));
    pass_pair(&mut game);
    let before = game.snapshot();
    activate(&mut game, tenderfoot, None);
    assert_eq!(game.state.players[0].mana[0], 7);
    pass_pair(&mut game);
    assert_eq!(game.object_view(tenderfoot).counters.plus_one, 1);
    let after = game.snapshot();
    game.restore(&before);
    activate(&mut game, tenderfoot, None);
    pass_pair(&mut game);
    assert_eq!(game.snapshot(), after);
}

#[test]
fn reanimation_targets_are_chosen_before_self_sacrifice() {
    let mut game = scenario();
    let faithful = named(
        &mut game,
        Player::P0,
        "Valgavoth's Faithful",
        Zone::Battlefield,
    );
    game.state.players[0].mana = [3, 0, 1, 0, 0];
    assert!(!has_activation(&game, faithful));
    let bears = spawn(&mut game, Player::P0, Card::GrizzlyBears, Zone::Graveyard);
    activate(&mut game, faithful, Some(Target::Card(bears)));
    assert_eq!(game.object_view(faithful).zone, Zone::Graveyard);
    pass_pair(&mut game);
    assert_eq!(game.object_view(bears).zone, Zone::Battlefield);
    assert!(game.object_view(bears).summoning_sick);
    assert_eq!(game.object_view(faithful).zone, Zone::Graveyard);
}

#[test]
fn noncreature_permanent_casts_enters_and_can_pay_tap_cost_immediately() {
    let mut game = scenario();
    let ramen = named(&mut game, Player::P0, "Instant Ramen", Zone::Hand);
    let (colored, generic) = game.state.objects[ramen as usize].card.cost();
    game.state.players[0].mana = colored;
    game.state.players[0].mana[0] += generic;
    cast(&mut game, ramen, None);
    pass_pair(&mut game);
    assert_eq!(game.object_view(ramen).zone, Zone::Battlefield);
    assert_eq!(game.state.stack.len(), 1); // Entry draw is an independent trigger.
    pass_pair(&mut game);
    assert_eq!(game.hand_count(Player::P0), 1);
    game.state.players[0].mana[0] = 2;
    activate(&mut game, ramen, None);
    assert_eq!(game.object_view(ramen).zone, Zone::Graveyard);
    assert_eq!(game.state.players[0].life, 20);
    pass_pair(&mut game);
    assert_eq!(game.state.players[0].life, 23);
}

#[test]
fn non_tap_ability_ignores_summoning_sickness_and_source_counter_cannot_follow_bounce() {
    let mut game = scenario();
    let tenderfoot = named(&mut game, Player::P0, "Intrepid Tenderfoot", Zone::Hand);
    game.move_object(tenderfoot, Zone::Battlefield);
    game.state.players[0].mana[0] = 3;
    assert!(game.object_view(tenderfoot).summoning_sick);
    activate(&mut game, tenderfoot, None);
    game.move_object(tenderfoot, Zone::Hand);
    game.move_object(tenderfoot, Zone::Battlefield);
    pass_pair(&mut game);
    assert_eq!(game.object_view(tenderfoot).counters.plus_one, 0);
}

#[test]
fn activated_draw_discard_suspends_and_restores_owned_resolution() {
    let mut game = scenario();
    let sailor = named(&mut game, Player::P0, "Strix Lookout", Zone::Battlefield);
    game.state.players[0].mana = [1, 1, 0, 0, 0];
    activate(&mut game, sailor, None);
    pass_pair(&mut game);
    assert!(matches!(game.state.pending, Pending::EffectDiscard { .. }));
    let before = game.snapshot();
    let discard = game
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::Discard { .. }))
        .unwrap();
    act(&mut game, discard.clone());
    assert_eq!(game.hand_count(Player::P0), 0);
    let after = game.snapshot();
    game.restore(&before);
    act(&mut game, discard);
    assert_eq!(game.snapshot(), after);
}

#[test]
fn entry_counters_and_any_kind_removal_present_distinct_exact_payments() {
    let mut game = scenario();
    let lamenter = named(&mut game, Player::P0, "Moonlit Lamenter", Zone::Hand);
    game.move_object(lamenter, Zone::Battlefield);
    assert_eq!(game.object_view(lamenter).counters.minus_one, 1);
    assert_eq!(
        (
            game.object_view(lamenter).power,
            game.object_view(lamenter).toughness
        ),
        (1, 4)
    );
    game.state.objects[lamenter as usize].counters.stun = 2;
    game.state.players[0].mana[0] = 2;
    let choices: Vec<_> = game
        .legal_actions()
        .into_iter()
        .filter(|action| matches!(action, Action::Activate { card, .. } if *card == lamenter))
        .collect();
    assert_eq!(choices.len(), 2);
    let remove_stun = choices.into_iter().find(|action|
        matches!(action, Action::Activate { counter_payment, .. } if counter_payment.stun == 1)).unwrap();
    let before = game.snapshot();
    act(&mut game, remove_stun.clone());
    assert_eq!(game.object_view(lamenter).counters.minus_one, 1);
    assert_eq!(game.object_view(lamenter).counters.stun, 1);
    assert_eq!(game.hand_count(Player::P0), 0);
    pass_pair(&mut game);
    assert_eq!(game.hand_count(Player::P0), 1);
    let after = game.snapshot();
    game.restore(&before);
    act(&mut game, remove_stun);
    pass_pair(&mut game);
    assert_eq!(game.snapshot(), after);
    game.move_object(lamenter, Zone::Hand);
    assert_eq!(game.object_view(lamenter).counters, Counters::default());
    game.move_object(lamenter, Zone::Battlefield);
    assert_eq!(game.object_view(lamenter).counters.minus_one, 1);
    assert_eq!(game.object_view(lamenter).counters.stun, 0);
}

#[test]
fn targeted_counter_cost_is_deferred_until_announcement_completes_and_replays() {
    let mut game = scenario();
    let elm = named(&mut game, Player::P0, "Gnarlbark Elm", Zone::Hand);
    game.move_object(elm, Zone::Battlefield);
    let bears = spawn(&mut game, Player::P1, Card::GrizzlyBears, Zone::Battlefield);
    game.state.players[0].mana = [2, 0, 1, 0, 0];
    let begin = game
        .legal_actions()
        .into_iter()
        .find(|action| matches!(action, Action::BeginActivate { card, .. } if *card == elm))
        .unwrap();
    act(&mut game, begin);
    assert_eq!(game.state.pending, Pending::Targets { player: Player::P0 });
    assert_eq!(game.state.players[0].mana, [2, 0, 1, 0, 0]);
    assert_eq!(game.object_view(elm).counters.minus_one, 2);
    assert!(game.state.stack.is_empty());
    let before = game.snapshot();
    let choose = Action::ChooseTarget {
        target: Target::Card(bears),
    };
    act(&mut game, choose.clone());
    assert_eq!(game.state.players[0].mana, [0; 5]);
    assert_eq!(game.object_view(elm).counters.minus_one, 0);
    assert_eq!(game.object_view(bears).zone, Zone::Battlefield);
    pass_pair(&mut game);
    assert_eq!(game.object_view(bears).zone, Zone::Graveyard);
    let after = game.snapshot();
    game.restore(&before);
    act(&mut game, choose);
    pass_pair(&mut game);
    assert_eq!(game.snapshot(), after);
}

#[test]
fn once_per_turn_life_and_typed_counter_cost_constraints_are_independent() {
    let mut game = scenario();
    let card = named(&mut game, Player::P0, "Moonlit Lamenter", Zone::Battlefield);
    let mut ability = game.state.objects[card as usize].card.activations()[0];
    ability.cost.remove = Some((Some(CounterKind::Stun), 2));
    ability.cost.life = 3;
    ability.once_per_turn = true;
    game.state.objects[card as usize].counters.stun = 2;
    game.state.players[0].life = 3;
    assert!(game.activation_cost_available(card, &ability, 0, Player::P0, true));
    game.state.players[0].life = 2;
    assert!(!game.activation_cost_available(card, &ability, 0, Player::P0, true));
    game.state.players[0].life = 3;
    game.state.objects[card as usize].counters.stun = 1;
    assert!(!game.activation_cost_available(card, &ability, 0, Player::P0, true));
    game.state.objects[card as usize].counters.stun = 2;
    game.state.objects[card as usize].used_abilities = 1;
    assert!(!game.activation_cost_available(card, &ability, 0, Player::P0, true));
    game.begin_turn();
    assert!(game.activation_cost_available(card, &ability, 0, Player::P0, true));
}

#[test]
fn mixed_counter_payments_are_complete_unique_and_forgery_is_atomic() {
    let mut game = scenario();
    let elm = named(&mut game, Player::P0, "Gnarlbark Elm", Zone::Hand);
    game.move_object(elm, Zone::Battlefield);
    game.state.objects[elm as usize].counters.stun = 2;
    game.state.players[0].mana = [2, 0, 1, 0, 0];
    let choices: Vec<_> = game
        .legal_actions()
        .into_iter()
        .filter(|action| matches!(action, Action::BeginActivate { card, .. } if *card == elm))
        .collect();
    let mut allocations: Vec<_> = choices
        .iter()
        .map(|action| {
            let Action::BeginActivate {
                counter_payment, ..
            } = action
            else {
                unreachable!()
            };
            (counter_payment.minus_one, counter_payment.stun)
        })
        .collect();
    allocations.sort();
    assert_eq!(allocations, vec![(0, 2), (1, 1), (2, 0)]);
    let before = game.snapshot();
    assert_eq!(
        game.apply(
            Player::P0,
            Action::BeginActivate {
                card: elm,
                ability: 0,
                payment: [2, 0, 1, 0, 0],
                counter_payment: Counters {
                    plus_one: 1,
                    minus_one: 1,
                    stun: 0
                }
            }
        ),
        Err(Error::IllegalAction)
    );
    assert_eq!(game.snapshot(), before);
    let mixed = choices.into_iter().find(|action|
        matches!(action, Action::BeginActivate { counter_payment, .. } if counter_payment.stun == 1)).unwrap();
    act(&mut game, mixed);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(elm),
        },
    );
    assert_eq!(
        game.object_view(elm).counters,
        Counters {
            plus_one: 0,
            minus_one: 1,
            stun: 1
        }
    );
    assert_eq!(
        (game.object_view(elm).power, game.object_view(elm).toughness),
        (2, 3)
    );
    pass_pair(&mut game);
    assert_eq!(
        (game.object_view(elm).power, game.object_view(elm).toughness),
        (0, 1)
    );
}
