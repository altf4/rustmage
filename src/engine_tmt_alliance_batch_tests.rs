//! Paid reflexive, optional, last-known and Ninja-creature trigger interactions.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, setup};
use super::*;

fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn finish(g: &mut Game) {
    let initial = g.state.announcing.as_ref().map(|a| a.item.id);
    for _ in 0..12 {
        if g.state.announcing.is_none() || g.state.announcing.as_ref().map(|a| a.item.id) != initial
        {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::PayCosts { .. }
                        | Action::FinishTargets
                        | Action::ChooseSpellCost { .. }
                        | Action::ChooseX { .. }
                )
            })
            .expect("payment");
        checked(g, a);
    }
    panic!("payment");
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    checked(g, Action::AnnounceCast { card: id });
    finish(g);
    id
}
fn refill(g: &mut Game) {
    for state in &mut g.state.players {
        state.mana = [30; 6];
    }
}
fn to_step(g: &mut Game, wanted: Step) {
    for _ in 0..100 {
        if g.state.step == wanted {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                )
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        checked(g, a);
    }
    panic!("step {wanted:?}");
}
fn mature(g: &mut Game) {
    next_turn(g);
    next_turn(g);
    to_step(g, Step::Main1);
    refill(g);
}
fn until(g: &mut Game, predicate: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..80 {
        if predicate(g) {
            return;
        }
        let a = if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            g.legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap()
        };
        checked(g, a);
    }
    panic!("phase {:?}", g.state.pending);
}
fn optional(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectOptional { .. })
    });
}
fn enter(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    g.prepare_triggers();
    id
}
fn foods(g: &Game, p: Player) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.controller == p
                && o.zone == Zone::Battlefield
                && o.card == Card::from_name("@token/283fd83a9aec83a51b736926").unwrap()
        })
        .count()
}
fn mode(g: &mut Game, index: u16) {
    until(g, |g| {
        !g.observe(g.actor().unwrap()).mode_choices.is_empty()
    });
    checked(g, Action::ChooseMode { mode: index });
}
fn count(g: &Game, id: CardId) -> usize {
    g.state
        .stack
        .iter()
        .chain(&g.state.pending_triggers)
        .chain(&g.state.ordering_triggers)
        .filter(|s| s.card == id && matches!(s.kind, StackKind::TriggeredAbility { .. }))
        .count()
}
#[test]
fn tmt_alliance_courier_decline_has_food_without_search_or_shuffle() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let _ = paid(&mut g, p, "Courier of Comestibles");
        optional(&mut g);
        let library = g.state.players[p.index()].library.clone();
        let rng = g.state.rng;
        checked(&mut g, Action::ChooseOptional { accept: false });
        settle(&mut g);
        assert_eq!(foods(&g, p), 1);
        assert_eq!(g.state.players[p.index()].library, library);
        assert_eq!(g.state.rng, rng);
    }
}
#[test]
fn tmt_alliance_courier_private_qualified_search_success_or_legal_failure() {
    for p in [Player::P0, Player::P1] {
        for found in [false, true] {
            let mut g = setup(p);
            let food = named(&mut g, p, "Everything Pizza", Zone::Library);
            let foreign = named(&mut g, p.other(), "Everything Pizza", Zone::Library);
            paid(&mut g, p, "Courier of Comestibles");
            optional(&mut g);
            checked(&mut g, Action::ChooseOptional { accept: true });
            assert_eq!(g.state.pending, Pending::EffectSearch { player: p });
            assert!(g.observe(p.other()).search_choice.is_none());
            reject(&mut g, Action::ChooseSearch { card: foreign });
            if found {
                checked(&mut g, Action::ChooseSearch { card: food });
            }
            checked(&mut g, Action::FinishSearch);
            settle(&mut g);
            assert_eq!(foods(&g, p), usize::from(!found));
            assert_eq!(
                g.state.objects[food as usize].zone,
                if found { Zone::Hand } else { Zone::Library }
            );
            if found {
                assert!(
                    serde_json::to_string(&g.observe(p.other()))
                        .unwrap()
                        .contains("Everything Pizza")
                );
            }
        }
    }
}
#[test]
fn tmt_alliance_courier_food_creature_and_source_departure_keep_search() {
    let p = Player::P0;
    let mut g = setup(p);
    let kitty = named(&mut g, p, "Ice Cream Kitty", Zone::Library);
    let courier = paid(&mut g, p, "Courier of Comestibles");
    until(&mut g, |g| {
        count(g, courier) > 0 && g.state.pending == Pending::Priority
    });
    g.move_object(courier, Zone::Graveyard);
    optional(&mut g);
    checked(&mut g, Action::ChooseOptional { accept: true });
    checked(&mut g, Action::ChooseSearch { card: kitty });
    checked(&mut g, Action::FinishSearch);
    settle(&mut g);
    assert_eq!(g.state.objects[kitty as usize].zone, Zone::Hand);
    assert_eq!(foods(&g, p), 0);
}
fn pizza_activation(
    g: &mut Game,
    p: Player,
    pizza: CardId,
    life: Player,
    damage: Target,
    counters: Option<CardId>,
) {
    g.state.priority = p;
    checked(
        g,
        Action::AnnounceActivate {
            card: pizza,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Player(life),
        },
    );
    checked(g, Action::ChooseTarget { target: damage });
    if let Some(id) = counters {
        checked(
            g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    finish(g);
}
#[test]
fn tmt_alliance_everything_pizza_paid_etb_basic_search_and_all_activation_clauses() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "Plains", Zone::Library);
        let nonbasic = named(&mut g, p, "Azorius Guildgate", Zone::Library);
        let pizza = paid(&mut g, p, "Everything Pizza");
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        reject(&mut g, Action::ChooseSearch { card: nonbasic });
        checked(&mut g, Action::ChooseSearch { card: land });
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.state.objects[land as usize].zone, Zone::Hand);
        let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        let discard = named(&mut g, p.other(), "Island", Zone::Hand);
        let life = g.state.players[p.index()].life;
        let hand = g.hand_count(p);
        let enemy = g.state.players[p.other().index()].life;
        pizza_activation(&mut g, p, pizza, p, Target::Player(p.other()), Some(bear));
        assert_eq!(g.state.objects[pizza as usize].zone, Zone::Graveyard);
        until(&mut g, |g| {
            g.state
                .resolving
                .as_ref()
                .is_some_and(|r| r.discards[p.other().index()] > 0)
        });
        assert!(g.observe(p).effect_choice.is_none());
        reject(&mut g, Action::Discard { card: land });
        checked(&mut g, Action::Discard { card: discard });
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life + 3);
        assert_eq!(g.hand_count(p), hand + 1);
        assert_eq!(g.state.players[p.other().index()].life, enemy - 3);
        assert_eq!(g.counter_amounts(bear).get(CounterKind::PlusOne), 3);
    }
}
#[test]
fn tmt_alliance_everything_pizza_optional_counter_target_and_partly_illegal_damage() {
    let p = Player::P0;
    let mut g = setup(p);
    let pizza = paid(&mut g, p, "Everything Pizza");
    settle(&mut g);
    let bear = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let life = g.state.players[p.other().index()].life;
    let hand = g.hand_count(p.other());
    pizza_activation(&mut g, p, pizza, p.other(), Target::Card(bear), None);
    g.move_object(bear, Zone::Hand);
    settle(&mut g);
    assert_eq!(g.state.players[p.other().index()].life, life + 3);
    assert_eq!(g.hand_count(p.other()), hand + 1);
}
#[test]
fn tmt_alliance_wingnut_all_three_choices_are_at_resolution_and_expire() {
    for p in [Player::P0, Player::P1] {
        for selected in 0..3 {
            let mut g = setup(p);
            let wing = paid(&mut g, p, "Wingnut, Bat on the Belfry");
            settle(&mut g);
            let foreign = enter(&mut g, p.other(), "Grizzly Bears");
            assert_eq!(count(&g, wing), 0);
            assert!(g.on_battlefield(foreign));
            enter(&mut g, p, "Grizzly Bears");
            assert!(g.observe(p).mode_choices.is_empty());
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectKeywordChoice { .. })
            });
            assert_eq!(
                g.observe(p)
                    .mode_choices
                    .iter()
                    .map(|m| m.text)
                    .collect::<Vec<_>>(),
                vec!["Flying", "Menace", "Haste"]
            );
            assert!(g.observe(p.other()).mode_choices.is_empty());
            reject(&mut g, Action::ChooseMode { mode: 3 });
            checked(
                &mut g,
                Action::ChooseMode {
                    mode: selected as u16,
                },
            );
            settle(&mut g);
            let keywords = [Keywords::FLYING, Keywords::MENACE, Keywords::HASTE];
            for (i, k) in keywords.into_iter().enumerate() {
                assert_eq!(g.keywords(wing).contains(k), i == selected);
            }
            next_turn(&mut g);
            assert!(!g.keywords(wing).contains(keywords[selected]));
        }
    }
}
#[test]
fn tmt_alliance_wingnut_attack_freezes_other_attacking_creatures() {
    let p = Player::P0;
    let mut g = setup(p);
    let wing = paid(&mut g, p, "Wingnut, Bat on the Belfry");
    settle(&mut g);
    let bear = enter(&mut g, p, "Grizzly Bears");
    settle(&mut g);
    mature(&mut g);
    to_step(&mut g, Step::Attackers);
    checked(&mut g, Action::Attack { card: wing });
    checked(&mut g, Action::Attack { card: bear });
    checked(&mut g, Action::FinishAttackers);
    settle(&mut g);
    assert_eq!(g.stats(wing).power, 1);
    assert_eq!(g.stats(bear).power, 3);
    let late = enter(&mut g, p, "Grizzly Bears");
    settle(&mut g);
    assert_eq!(g.stats(late).power, 2);
    next_turn(&mut g);
    assert_eq!(g.stats(bear).power, 2);
}
#[test]
fn tmt_alliance_lita_modes_reserved_before_resolution_and_reset_each_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let lita = paid(&mut g, p, "Lita, Little Orphan Amphibian");
        settle(&mut g);
        for chosen in 0..3 {
            enter(&mut g, p, "Grizzly Bears");
            mode(&mut g, chosen);
            reject(&mut g, Action::ChooseMode { mode: chosen });
            settle(&mut g);
        }
        assert_eq!(g.counter_amounts(lita).get(CounterKind::PlusOne), 1);
        assert_eq!(foods(&g, p), 1);
        assert_eq!(g.observe(p).modal_history[0].modes, vec![0, 1, 2]);
        enter(&mut g, p, "Grizzly Bears");
        settle(&mut g);
        assert_eq!(g.counter_amounts(lita).get(CounterKind::PlusOne), 1);
        next_turn(&mut g);
        enter(&mut g, p, "Grizzly Bears");
        mode(&mut g, 0);
        settle(&mut g);
        assert_eq!(g.counter_amounts(lita).get(CounterKind::PlusOne), 2);
    }
}
#[test]
fn tmt_alliance_lita_departure_does_not_rebind_counter_to_new_incarnation() {
    let p = Player::P0;
    let mut g = setup(p);
    let lita = paid(&mut g, p, "Lita, Little Orphan Amphibian");
    settle(&mut g);
    enter(&mut g, p, "Grizzly Bears");
    mode(&mut g, 0);
    g.move_object(lita, Zone::Hand);
    g.move_object(lita, Zone::Battlefield);
    settle(&mut g);
    assert_eq!(g.counter_amounts(lita).get(CounterKind::PlusOne), 0);
    enter(&mut g, p, "Grizzly Bears");
    mode(&mut g, 0);
    settle(&mut g);
    assert_eq!(g.counter_amounts(lita).get(CounterKind::PlusOne), 1);
}
#[test]
fn tmt_alliance_madame_optional_payment_current_power_and_control() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let madame = paid(&mut g, p, "Madame Null, Power Broker");
            settle(&mut g);
            assert!(g.keywords(madame).contains(Keywords::DEATHTOUCH));
            enter(&mut g, p.other(), "Grizzly Bears");
            assert_eq!(count(&g, madame), 0);
            let bear = enter(&mut g, p, "Grizzly Bears");
            let source = g.object_ref(madame);
            g.grant_keyword_effects(&[bear], source, Keywords::HEXPROOF);
            g.state.objects[bear as usize].growth.power = 3;
            let life = g.state.players[p.index()].life;
            optional(&mut g);
            checked(&mut g, Action::ChooseOptional { accept });
            settle(&mut g);
            assert_eq!(
                g.state.players[p.index()].life,
                life - if accept { 5 } else { 0 }
            );
            assert_eq!(
                g.counter_amounts(bear).get(CounterKind::PlusOne),
                if accept { 5 } else { 0 }
            );
        }
    }
}
#[test]
fn tmt_alliance_madame_unaffordable_and_zero_negative_power_payments() {
    for power in [-2, 0, 5] {
        let p = Player::P0;
        let mut g = setup(p);
        let madame = paid(&mut g, p, "Madame Null, Power Broker");
        settle(&mut g);
        g.state.players[p.index()].life = 2;
        let bear = enter(&mut g, p, "Aegis Turtle");
        g.state.objects[bear as usize].growth.power = power;
        if power > 2 {
            settle(&mut g);
        } else {
            optional(&mut g);
            checked(&mut g, Action::ChooseOptional { accept: true });
            settle(&mut g);
        }
        assert_eq!(g.state.players[p.index()].life, 2);
        assert_eq!(g.counter_amounts(bear).get(CounterKind::PlusOne), 0);
        assert!(g.on_battlefield(madame));
    }
}
#[test]
fn tmt_alliance_madame_source_departure_and_subject_last_known_identity() {
    let p = Player::P0;
    let mut g = setup(p);
    let madame = paid(&mut g, p, "Madame Null, Power Broker");
    settle(&mut g);
    let bear = enter(&mut g, p, "Grizzly Bears");
    until(&mut g, |g| {
        g.state.pending == Pending::Priority && count(g, madame) > 0
    });
    g.move_object(madame, Zone::Graveyard);
    g.move_object(bear, Zone::Hand);
    g.move_object(bear, Zone::Battlefield);
    let life = g.state.players[p.index()].life;
    optional(&mut g);
    checked(&mut g, Action::ChooseOptional { accept: true });
    settle(&mut g);
    assert_eq!(g.state.players[p.index()].life, life - 2);
    assert_eq!(g.counter_amounts(bear).get(CounterKind::PlusOne), 0);
}
#[test]
fn tmt_alliance_triceraton_xx_payment_and_reanimation_x_zero() {
    for p in [Player::P0, Player::P1] {
        for x in [0, 2] {
            let mut g = setup(p);
            g.state.players[p.index()].mana = [2, 0, 0, 0, 0, 4];
            let id = named(&mut g, p, "Triceraton Commander", Zone::Hand);
            let before = g.state.players[p.index()].mana;
            checked(&mut g, Action::AnnounceCast { card: id });
            checked(&mut g, Action::ChooseX { value: x });
            finish(&mut g);
            assert_eq!(
                before.iter().sum::<u32>() - g.state.players[p.index()].mana.iter().sum::<u32>(),
                2 + 2 * x
            );
            assert_eq!(
                g.state.stack.last().unwrap().mana_value(),
                2 + 2 * u64::from(x)
            );
            settle(&mut g);
            let tokens: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(_, o)| {
                    o.card == Card::from_name("@token/dinosaur_soldier_2_2_white").unwrap()
                        && o.zone == Zone::Battlefield
                })
                .map(|(i, _)| i as CardId)
                .collect();
            assert_eq!(tokens.len(), x as usize);
            for id in tokens {
                assert_eq!(
                    g.stats(id),
                    Stats {
                        power: 2,
                        toughness: 2
                    }
                );
                assert_eq!(g.object_view(id).colors, 1);
                assert!(g.has_subtype(id, "Soldier"));
            }
            g.move_object(id, Zone::Graveyard);
            g.move_object(id, Zone::Battlefield);
            settle(&mut g);
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.card
                        == Card::from_name("@token/dinosaur_soldier_2_2_white").unwrap()
                        && o.zone == Zone::Battlefield)
                    .count(),
                x as usize
            );
        }
    }
}
#[test]
fn tmt_alliance_triceraton_attack_buffs_nonattacking_own_dinosaurs_only() {
    let p = Player::P0;
    let mut g = setup(p);
    let commander = paid(&mut g, p, "Triceraton Commander"); // Choose X explicitly below in this scenario.
    settle(&mut g);
    let own = enter(&mut g, p, "Belligerent Yearling");
    let enemy = enter(&mut g, p.other(), "Belligerent Yearling");
    let bear = enter(&mut g, p, "Grizzly Bears");
    settle(&mut g);
    mature(&mut g);
    to_step(&mut g, Step::Attackers);
    checked(&mut g, Action::Attack { card: commander });
    checked(&mut g, Action::FinishAttackers);
    settle(&mut g);
    assert_eq!(g.stats(own).power, 4);
    assert!(g.keywords(own).contains(Keywords::FLYING));
    assert_eq!(g.stats(enemy).power, 3);
    assert!(!g.keywords(enemy).contains(Keywords::FLYING));
    assert_eq!(g.stats(bear).power, 2);
    assert_eq!(g.stats(commander).power, 2);
    next_turn(&mut g);
    assert_eq!(g.stats(own).power, 3);
    assert!(!g.keywords(own).contains(Keywords::FLYING));
}
#[test]
fn tmt_alliance_tokka_uncounterable_hybrid_menace_and_normal_casts_no_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tokka = paid(&mut g, p, "Tokka & Rahzar, Terrible Twos");
        let spell = g.state.stack.last().unwrap();
        assert!(!g.stack_item_is_counterable(spell));
        settle(&mut g);
        assert!(g.keywords(tokka).contains(Keywords::MENACE));
        paid(&mut g, p, "Grizzly Bears");
        assert_eq!(count(&g, tokka), 0);
        settle(&mut g);
    }
}
#[test]
fn tmt_alliance_tokka_actual_discounted_payment_hits_casting_player_even_after_source_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tokka = paid(&mut g, p, "Tokka & Rahzar, Terrible Twos");
        settle(&mut g);
        let reducer = enter(&mut g, p, "Mocking Sprite");
        settle(&mut g);
        let shock = named(&mut g, p, "Lightning Strike", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: shock });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        finish(&mut g);
        assert_eq!(count(&g, tokka), 1);
        let life = g.state.players[p.index()].life;
        g.move_object(tokka, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, life - 3);
        assert!(g.on_battlefield(reducer));
    }
}
#[test]
fn tmt_alliance_april_records_earlier_casts_distinct_types_and_response_spells() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid(&mut g, p, "Grizzly Bears");
        settle(&mut g);
        paid(&mut g, p, "Hedron Archive");
        settle(&mut g);
        let april = paid(&mut g, p, "April O'Neil, Hacktivist");
        settle(&mut g);
        paid(&mut g, p, "Grizzly Bears");
        settle(&mut g);
        let land = named(&mut g, p, "Plains", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: land });
        to_step(&mut g, Step::End);
        until(&mut g, |g| {
            g.state.pending == Pending::Priority && count(g, april) > 0
        });
        refill(&mut g);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Player(p.other())]);
        let before = g.hand_count(p);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 3);
        let history = g.observe(p).players[p.index()].cast_card_types_this_turn;
        assert_eq!(history, 1 | 16 | 8);
        next_turn(&mut g);
        assert_eq!(g.observe(p).players[p.index()].cast_card_types_this_turn, 0);
    }
}
#[test]
fn tmt_alliance_april_countered_spell_counts_and_source_departure_keeps_trigger() {
    let p = Player::P0;
    let mut g = setup(p);
    let enchantment = paid(&mut g, p, "Anthem of Champions");
    let stack = g.state.stack.last().unwrap().id;
    g.counter_stack_item(stack, false);
    settle(&mut g);
    assert_eq!(g.state.objects[enchantment as usize].zone, Zone::Graveyard);
    let april = paid(&mut g, p, "April O'Neil, Hacktivist");
    settle(&mut g);
    to_step(&mut g, Step::End);
    until(&mut g, |g| {
        g.state.pending == Pending::Priority && count(g, april) > 0
    });
    g.move_object(april, Zone::Hand);
    let before = g.hand_count(p);
    settle(&mut g);
    assert_eq!(g.hand_count(p), before + 2);
}
#[test]
fn tmt_alliance_tokka_paid_counterspell_fails_and_opponent_discount_is_not_targeting() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tokka = paid(&mut g, p, "Tokka & Rahzar, Terrible Twos");
        checked(&mut g, Action::Pass);
        let counter = named(&mut g, p.other(), "Essence Scatter", Zone::Hand);
        cast(&mut g, counter, None, &[Target::Spell(tokka)]);
        settle(&mut g);
        assert!(g.on_battlefield(tokka));
        assert_eq!(g.state.objects[counter as usize].zone, Zone::Graveyard);
        enter(&mut g, p.other(), "Mocking Sprite");
        settle(&mut g);
        if g.actor() != Some(p.other()) {
            checked(&mut g, Action::Pass);
        }
        let spell = named(&mut g, p.other(), "Lightning Strike", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Player(p)]);
        assert_eq!(count(&g, tokka), 1);
        assert!(
            g.state
                .pending_triggers
                .iter()
                .chain(&g.state.stack)
                .filter(|s| s.card == tokka)
                .all(|s| s.targets.is_empty())
        );
        let before = g.state.players[p.other().index()].life;
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, before - 3);
    }
}
#[test]
fn tmt_alliance_lita_ninja_doubling_reserves_distinct_modes_before_either_resolves() {
    let p = Player::P0;
    let mut g = setup(p);
    paid(&mut g, p, "Splinter, Radical Rat");
    settle(&mut g);
    let lita = paid(&mut g, p, "Lita, Little Orphan Amphibian");
    settle(&mut g);
    enter(&mut g, p, "Grizzly Bears");
    assert_eq!(count(&g, lita), 2);
    mode(&mut g, 0);
    mode(&mut g, 1);
    settle(&mut g);
    assert_eq!(foods(&g, p), 1);
    assert_eq!(g.counter_amounts(lita).get(CounterKind::PlusOne), 1);
    assert_eq!(g.observe(p).modal_history[0].modes, vec![0, 1]);
}
#[test]
fn tmt_alliance_wingnut_departed_source_does_not_offer_keyword_choice_for_returned_object() {
    let p = Player::P0;
    let mut g = setup(p);
    let wing = paid(&mut g, p, "Wingnut, Bat on the Belfry");
    settle(&mut g);
    enter(&mut g, p, "Grizzly Bears");
    assert_eq!(count(&g, wing), 1);
    g.move_object(wing, Zone::Hand);
    g.move_object(wing, Zone::Battlefield);
    settle(&mut g);
    for keyword in [Keywords::FLYING, Keywords::MENACE, Keywords::HASTE] {
        assert!(!g.keywords(wing).contains(keyword));
    }
    assert!(g.observe(p).mode_choices.is_empty());
}
