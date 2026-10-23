use super::tests::{act, cast, scenario, spawn};
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let c = Card::from_name(name).unwrap();
    assert!(c.is_supported(), "{name}");
    spawn(g, p, c, zone)
}
fn funded(g: &mut Game, name: &str) -> CardId {
    let id = named(g, Player::P0, name, Zone::Hand);
    g.state.players[0].mana = [20; 6];
    id
}
fn replay(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    let p = g.actor().unwrap();
    let mut fork = g.clone();
    act(&mut fork, a.clone());
    act(g, a.clone());
    assert_eq!(fork.snapshot(), g.snapshot());
    let after = g.snapshot();
    g.restore(&before).unwrap();
    assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), after);
}
fn passes(g: &mut Game) {
    replay(g, Action::Pass);
    replay(g, Action::Pass);
}
fn trigger(g: &mut Game, targets: &[Target]) {
    g.prepare_triggers();
    let mut n = 0;
    for _ in 0..20 {
        if g.state.pending == Pending::Priority {
            return;
        }
        let actions = g.legal_actions();
        let a = if actions
            .iter()
            .any(|a| matches!(a, Action::ChooseTarget { .. }))
            && n < targets.len()
        {
            let a = Action::ChooseTarget { target: targets[n] };
            n += 1;
            a
        } else {
            actions
                .into_iter()
                .find(|a| matches!(a, Action::PutTrigger { .. } | Action::FinishTargets))
                .unwrap()
        };
        replay(g, a);
    }
    panic!("trigger announcement stalled");
}
fn enter(g: &mut Game, name: &str) -> CardId {
    let id = named(g, Player::P0, name, Zone::Hand);
    g.move_object(id, Zone::Battlefield);
    id
}
fn illegal(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}

fn tokens(g: &Game, subtype: &str) -> usize {
    g.state
        .objects
        .iter()
        .filter(|o| {
            o.zone == Zone::Battlefield
                && o.owner == Player::P0
                && o.card.is_token()
                && o.card.has_subtype(subtype)
        })
        .count()
}
fn attack(g: &mut Game, id: CardId) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    replay(g, Action::Attack { card: id });
    replay(g, Action::FinishAttackers);
}
fn mode(g: &mut Game, name: &str, index: u16, target: CardId) {
    let spell = funded(g, name);
    replay(g, Action::AnnounceCast { card: spell });
    replay(g, Action::ChooseMode { mode: index });
    replay(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    let pay = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(g, pay);
}
fn grave(g: &mut Game, permanents: bool, n: usize) -> Vec<CardId> {
    (0..n)
        .map(|_| {
            named(
                g,
                Player::P0,
                if permanents { "Plains" } else { "Shock" },
                Zone::Graveyard,
            )
        })
        .collect()
}
fn arrange(g: &mut Game, destination: LibraryDestination) {
    while g.observe(Player::P0).library_choice.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::ArrangeLibrary {destination:d,..} if *d==destination))
            .unwrap();
        replay(g, a);
    }
}

#[test]
fn donatello_false_event_cannot_retroactively_trigger_after_artifact_enters() {
    let mut g = scenario();
    enter(&mut g, "Donatello, Turtle Techie");
    trigger(&mut g, &[]);
    assert!(g.state.stack.is_empty());
    named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
    assert!(g.state.stack.is_empty());
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
}
#[test]
fn donatello_condition_is_rechecked_at_resolution_but_source_may_depart() {
    for remove_artifact in [false, true] {
        let mut g = scenario();
        let artifact = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
        let source = enter(&mut g, "Donatello, Turtle Techie");
        trigger(&mut g, &[]);
        assert_eq!(g.state.stack.len(), 1);
        g.move_object(source, Zone::Hand);
        if remove_artifact {
            g.move_object(artifact, Zone::Hand);
        }
        passes(&mut g);
        assert_eq!(
            g.state.players[0].cards_drawn_this_turn,
            u64::from(!remove_artifact)
        );
    }
}
#[test]
fn dwynens_elite_requires_another_elf_including_changeling_or_kindred() {
    for other in [None, Some("Three Tree Mascot"), Some("Morcant's Eyes")] {
        let mut g = scenario();
        if let Some(name) = other {
            named(&mut g, Player::P0, name, Zone::Battlefield);
        }
        enter(&mut g, "Dwynen's Elite");
        trigger(&mut g, &[]);
        if other.is_some() {
            passes(&mut g);
        } else {
            assert!(g.state.stack.is_empty());
        }
        assert_eq!(tokens(&g, "Elf"), usize::from(other.is_some()));
    }
}
#[test]
fn elite_simultaneous_entries_see_other_elves_and_still_recheck_live_board() {
    let mut g = scenario();
    let a = named(&mut g, Player::P0, "Dwynen's Elite", Zone::Hand);
    let b = named(&mut g, Player::P0, "Dwynen's Elite", Zone::Hand);
    let events = g.move_objects_raw(&[a, b], Zone::Battlefield);
    g.finish_zone_events(&events, &[]);
    trigger(&mut g, &[]);
    assert_eq!(g.state.stack.len(), 2);
    passes(&mut g);
    passes(&mut g);
    assert_eq!(tokens(&g, "Elf"), 2);
}
#[test]
fn elite_true_event_that_loses_its_other_elf_creates_no_token() {
    let mut g = scenario();
    let elf = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
    enter(&mut g, "Dwynen's Elite");
    trigger(&mut g, &[]);
    g.move_object(elf, Zone::Hand);
    passes(&mut g);
    assert_eq!(tokens(&g, "Elf"), 0);
}
#[test]
fn hero_checks_later_resolution_condition_after_unconditional_draw() {
    for other in [false, true] {
        let mut g = scenario();
        let hero = enter(&mut g, "Hero in Training");
        trigger(&mut g, &[]);
        if other {
            named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
        }
        passes(&mut g);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
        assert_eq!(g.state.players[0].life, if other { 22 } else { 20 });
        assert_eq!(g.object_view(hero).zone, Zone::Battlefield);
    }
}
#[test]
fn hero_gain_failure_rolls_back_earlier_draw_branch_and_histories() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
    enter(&mut g, "Hero in Training");
    trigger(&mut g, &[]);
    g.state.players[0].life = i128::MAX;
    replay(&mut g, Action::Pass);
    let before = g.snapshot();
    let rev = g.revision();
    assert!(matches!(
        g.apply(g.actor().unwrap(), Action::Pass),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
}
#[test]
fn threshold_attacker_checks_seven_cards_at_both_times_and_bonus_expires() {
    for count in [6, 7] {
        let mut g = scenario();
        let ids = grave(&mut g, false, count);
        let source = named(&mut g, Player::P0, "Crypt Feaster", Zone::Battlefield);
        assert!(g.keywords(source).contains(Keywords::MENACE));
        attack(&mut g, source);
        trigger(&mut g, &[]);
        if count == 6 {
            assert!(g.state.stack.is_empty());
            continue;
        }
        let saved = g.snapshot();
        passes(&mut g);
        assert_eq!(g.object_view(source).power, 5);
        g.restore(&saved).unwrap();
        g.move_object(ids[0], Zone::Exile);
        passes(&mut g);
        assert_eq!(g.object_view(source).power, 3);
    }
}
#[test]
fn descend_attackers_count_permanent_cards_not_instants_or_opponents_and_recheck() {
    for name in ["Malamet Veteran", "Stinging Cave Crawler"] {
        let mut g = scenario();
        let ids = grave(&mut g, true, 4);
        grave(&mut g, false, 3);
        let source = named(&mut g, Player::P0, name, Zone::Battlefield);
        let bear = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        attack(&mut g, source);
        let target = [Target::Card(bear)];
        trigger(
            &mut g,
            if name == "Malamet Veteran" {
                &target
            } else {
                &[]
            },
        );
        let saved = g.snapshot();
        passes(&mut g);
        if name == "Malamet Veteran" {
            assert!(g.keywords(source).contains(Keywords::TRAMPLE));
            assert_eq!(g.object_view(bear).counters.plus_one, 1);
        } else {
            assert!(g.keywords(source).contains(Keywords::DEATHTOUCH));
            assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
            assert_eq!(g.state.players[0].life, 19);
        }
        g.restore(&saved).unwrap();
        g.move_object(ids[0], Zone::Exile);
        passes(&mut g);
        assert_eq!(g.object_view(bear).counters.plus_one, 0);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 0);
        assert_eq!(g.state.players[0].life, 20);
    }
}
#[test]
fn descend_false_event_ignores_nonpermanents_and_opponents_graveyards() {
    for name in ["Malamet Veteran", "Stinging Cave Crawler"] {
        let mut g = scenario();
        grave(&mut g, true, 3);
        grave(&mut g, false, 7);
        named(&mut g, Player::P1, "Plains", Zone::Graveyard);
        let source = named(&mut g, Player::P0, name, Zone::Battlefield);
        attack(&mut g, source);
        trigger(&mut g, &[]);
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn ferocious_includes_self_but_nessian_requires_another_qualifying_creature() {
    for name in ["Nasty Little Rabbit", "Nessian Hornbeetle"] {
        let mut g = scenario();
        let source = named(&mut g, Player::P0, name, Zone::Battlefield);
        g.state.objects[source as usize].counters.plus_one = 3;
        g.begin_step(Step::BeginCombat);
        g.give_priority();
        trigger(&mut g, &[]);
        if name == "Nasty Little Rabbit" {
            passes(&mut g);
            assert_eq!(g.object_view(source).counters.plus_one, 4);
        } else {
            assert!(g.state.stack.is_empty());
            let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
            g.state.objects[bear as usize].growth = 2.into();
            g.begin_step(Step::BeginCombat);
            g.give_priority();
            trigger(&mut g, &[]);
            passes(&mut g);
            assert_eq!(g.object_view(source).counters.plus_one, 4);
        }
    }
}
#[test]
fn step_intervening_if_power_drops_before_resolution_and_opponents_turn_never_triggers() {
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Nessian Hornbeetle", Zone::Battlefield);
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.state.objects[bear as usize].growth = 2.into();
    g.begin_step(Step::BeginCombat);
    g.give_priority();
    trigger(&mut g, &[]);
    g.state.objects[bear as usize].growth = Stats::default();
    passes(&mut g);
    assert_eq!(g.object_view(source).counters.plus_one, 0);
    g.state.active = Player::P1;
    g.state.objects[bear as usize].growth = 2.into();
    g.begin_step(Step::BeginCombat);
    g.give_priority();
    trigger(&mut g, &[]);
    assert!(g.state.stack.is_empty());
}
#[test]
fn rabbit_response_checks_condition_after_growth_and_keeps_private_scry_owned() {
    for rabbit in [false, true] {
        let mut g = scenario();
        let bear = named(
            &mut g,
            Player::P0,
            if rabbit {
                "Three Tree Mascot"
            } else {
                "Bear Cub"
            },
            Zone::Battlefield,
        );
        let old = g.object_view(bear);
        let spell = funded(&mut g, "Rabbit Response");
        cast(&mut g, spell, None);
        passes(&mut g);
        assert_eq!(g.object_view(bear).power, old.power + 2);
        assert_eq!(g.object_view(bear).toughness, old.toughness + 1);
        assert_eq!(g.observe(Player::P0).library_choice.is_some(), rabbit);
        if rabbit {
            assert!(g.observe(Player::P1).library_choice.is_none());
            arrange(&mut g, LibraryDestination::Top);
            assert_eq!(g.object_view(bear).power, old.power + 2);
        }
    }
}
#[test]
fn failed_fording_uses_live_desert_count_then_surveils_privately_without_repeating_bounce() {
    let mut g = scenario();
    let victim = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let spell = funded(&mut g, "Failed Fording");
    cast(&mut g, spell, Some(Target::Card(victim)));
    let desert = Card::catalogue()
        .find(|c| {
            c.is_supported() && c.is_standard_legal() && c.is_land() && c.has_subtype("Desert")
        })
        .unwrap();
    spawn(&mut g, Player::P0, desert, Zone::Battlefield);
    passes(&mut g);
    assert_eq!(g.object_view(victim).zone, Zone::Hand);
    assert!(g.observe(Player::P1).library_choice.is_none());
    arrange(&mut g, LibraryDestination::Graveyard);
    assert_eq!(g.object_view(victim).zone, Zone::Hand);
}
#[test]
fn taken_by_nightmares_sees_its_own_removal_before_enchantment_condition() {
    for victim_name in ["Bear Cub", "Grasping Longneck"] {
        let mut g = scenario();
        let victim = named(&mut g, Player::P0, victim_name, Zone::Battlefield);
        let spell = funded(&mut g, "Taken by Nightmares");
        cast(&mut g, spell, Some(Target::Card(victim)));
        passes(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Exile);
        assert!(g.observe(Player::P0).library_choice.is_none());
    }
    let mut g = scenario();
    named(&mut g, Player::P0, "Clachan Festival", Zone::Battlefield);
    let victim = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let spell = funded(&mut g, "Taken by Nightmares");
    cast(&mut g, spell, Some(Target::Card(victim)));
    passes(&mut g);
    assert!(g.observe(Player::P0).library_choice.is_some());
    arrange(&mut g, LibraryDestination::Top);
}
#[test]
fn violent_urge_checks_distinct_graveyard_types_before_spell_itself_arrives() {
    for delirium in [false, true] {
        let mut g = scenario();
        for name in ["Plains", "Three Tree Mascot"] {
            named(&mut g, Player::P0, name, Zone::Graveyard);
        }
        if delirium {
            named(&mut g, Player::P0, "Clachan Festival", Zone::Graveyard);
        }
        let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let spell = funded(&mut g, "Violent Urge");
        cast(&mut g, spell, Some(Target::Card(bear)));
        passes(&mut g);
        assert_eq!(g.object_view(bear).power, 3);
        assert!(g.keywords(bear).contains(Keywords::FIRST_STRIKE));
        assert_eq!(g.keywords(bear).contains(Keywords::DOUBLE_STRIKE), delirium);
        g.finish_cleanup();
        assert_eq!(g.object_view(bear).power, 2);
        assert!(!g.keywords(bear).contains(Keywords::FIRST_STRIKE));
    }
}
#[test]
fn youre_not_alone_replaces_bonus_and_reads_creature_count_at_resolution() {
    for extra in [0, 2] {
        let mut g = scenario();
        let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let spell = funded(&mut g, "You're Not Alone");
        cast(&mut g, spell, Some(Target::Card(bear)));
        for _ in 0..extra {
            named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
        }
        passes(&mut g);
        assert_eq!(
            (g.object_view(bear).power, g.object_view(bear).toughness),
            if extra == 0 { (4, 4) } else { (6, 6) }
        );
    }
}
#[test]
fn snakeskin_veil_adds_counter_then_hexproof_and_rejects_enemy_target_actions() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let spell = funded(&mut g, "Snakeskin Veil");
    cast(&mut g, spell, Some(Target::Card(bear)));
    passes(&mut g);
    assert_eq!(g.object_view(bear).counters.plus_one, 1);
    assert!(g.keywords(bear).contains(Keywords::HEXPROOF));
    replay(&mut g, Action::Pass);
    let shock = named(&mut g, Player::P1, "Shock", Zone::Hand);
    g.state.players[1].mana = [20; 6];
    let mut forged = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::Cast{card,target:Some(Target::Player(_)),..}if *card==shock))
        .unwrap();
    if let Action::Cast { target, .. } = &mut forged {
        *target = Some(Target::Card(bear));
    }
    illegal(&mut g, forged);
}
#[test]
fn spectacular_warg_and_valor_modes_keep_complete_targets_bodies_and_keywords() {
    for (name, positive) in [
        ("Moment of Valor", 0),
        ("Spectacular Tactics", 0),
        ("Warg Tactics", 1),
    ] {
        for index in [0, 1] {
            let mut g = scenario();
            let own = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
            let enemy = named(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
            g.state.objects[own as usize].tapped = true;
            mode(
                &mut g,
                name,
                index,
                if index == positive { own } else { enemy },
            );
            passes(&mut g);
            if index == positive {
                if name == "Moment of Valor" {
                    assert!(!g.object_view(own).tapped);
                    assert_eq!(g.object_view(own).power, 3);
                    assert!(g.keywords(own).contains(Keywords::INDESTRUCTIBLE));
                } else {
                    assert_eq!(g.object_view(own).counters.plus_one, 1);
                    assert!(g.keywords(own).contains(Keywords::HEXPROOF));
                    if name == "Warg Tactics" {
                        assert!(g.keywords(own).contains(Keywords::TRAMPLE));
                    }
                }
            } else {
                assert_eq!(g.object_view(enemy).zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn rosa_reach_combat_counter_and_temporary_lifelink_survive_owned_replay() {
    let mut g = scenario();
    let source = named(
        &mut g,
        Player::P0,
        "Rosa, Resolute White Mage",
        Zone::Battlefield,
    );
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    assert!(g.keywords(source).contains(Keywords::REACH));
    g.begin_step(Step::BeginCombat);
    g.give_priority();
    g.prepare_triggers();
    let enemy = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    illegal(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(enemy),
        },
    );
    trigger(&mut g, &[Target::Card(bear)]);
    passes(&mut g);
    assert_eq!(g.object_view(bear).counters.plus_one, 1);
    assert!(g.keywords(bear).contains(Keywords::LIFELINK));
    g.finish_cleanup();
    assert_eq!(g.object_view(bear).counters.plus_one, 1);
    assert!(!g.keywords(bear).contains(Keywords::LIFELINK));
}
#[test]
fn wrapped_conditions_preserve_event_and_limit_metadata_without_consuming_false_event() {
    const C: TriggerCondition = TriggerCondition::InterveningIf {
        guard: StepGuard::Threshold {
            value: StepValue::Creatures,
            minimum: Some(2),
            maximum: None,
        },
        event: &TriggerCondition::OncePerTurn(&TriggerCondition::SelfEnters),
    };
    let mut g = scenario();
    let source = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    assert_eq!(C.event(), TriggerCondition::SelfEnters);
    assert!(C.once_per_turn());
    assert!(!g.trigger_guards_hold(C, Player::P0, g.object_ref(source), false));
    assert!(g.state.triggered_this_turn.is_empty());
    named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    assert!(g.trigger_guards_hold(C, Player::P0, g.object_ref(source), false));
}

#[test]
fn trash_conditional_optional_discard_and_draw_resume_without_repeating_damage() {
    for (raccoon, accept) in [(false, false), (true, false), (true, true)] {
        let mut g = scenario();
        if raccoon {
            named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
        }
        let card = named(&mut g, Player::P0, "Forest", Zone::Hand);
        let victim = named(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
        let spell = funded(&mut g, "Take Out the Trash");
        cast(&mut g, spell, Some(Target::Card(victim)));
        passes(&mut g);
        assert_eq!(g.object_view(victim).damage, 3);
        assert_eq!(g.observe(Player::P0).effect_choice.is_some(), raccoon);
        if raccoon {
            assert!(g.observe(Player::P1).effect_choice.is_none());
            illegal(&mut g, Action::Pass);
            replay(
                &mut g,
                if accept {
                    Action::Discard { card }
                } else {
                    Action::DeclineEffectPayment
                },
            );
        }
        assert_eq!(g.object_view(victim).damage, 3);
        assert_eq!(g.state.players[0].cards_drawn_this_turn, u64::from(accept));
        assert_eq!(
            g.object_view(card).zone,
            if accept { Zone::Graveyard } else { Zone::Hand }
        );
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    }
}
#[test]
fn lethally_damaged_raccoon_still_satisfies_condition_until_spell_finishes() {
    let mut g = scenario();
    let victim = named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
    let spell = funded(&mut g, "Take Out the Trash");
    let card = named(&mut g, Player::P0, "Forest", Zone::Hand);
    cast(&mut g, spell, Some(Target::Card(victim)));
    passes(&mut g);
    assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
    assert_eq!(g.object_view(victim).damage, 3);
    assert!(g.observe(Player::P0).effect_choice.is_some());
    replay(&mut g, Action::Discard { card });
    assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
}
#[test]
fn newly_illegal_target_fizzles_the_whole_spell_including_true_conditional_scry() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Clachan Festival", Zone::Battlefield);
    let victim = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let spell = funded(&mut g, "Taken by Nightmares");
    cast(&mut g, spell, Some(Target::Card(victim)));
    g.state.objects[victim as usize].gained_keywords = Keywords::HEXPROOF;
    passes(&mut g);
    assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
    assert!(g.observe(Player::P0).library_choice.is_none());
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
}
#[test]
fn flat_if_else_offsets_and_scry_pc_keep_common_tail_outside_both_branches() {
    // Synthetic IR exercises block scope/continuations without admitting a fictional card.
    const TRUE: &[Instruction] = &[
        Instruction::If {
            guard: StepGuard::Always,
            skip: 3,
        },
        Instruction::Scry(1),
        Instruction::GainLife(PlayerSet::Controller, 2),
        Instruction::Skip(1),
        Instruction::GainLife(PlayerSet::Controller, 4),
        Instruction::Draw(PlayerSet::Controller, 1),
    ];
    const FALSE: &[Instruction] = &[
        Instruction::If {
            guard: StepGuard::Threshold {
                value: StepValue::Creatures,
                minimum: Some(1),
                maximum: None,
            },
            skip: 3,
        },
        Instruction::Scry(1),
        Instruction::GainLife(PlayerSet::Controller, 2),
        Instruction::Skip(1),
        Instruction::GainLife(PlayerSet::Controller, 4),
        Instruction::Draw(PlayerSet::Controller, 1),
    ];
    for (condition, program) in [(true, TRUE), (false, FALSE)] {
        let mut g = scenario();
        let source = funded(&mut g, "Shock");
        let item = g.cast_item(Player::P0, source, [0; 6]);
        g.state.resolving = Some(Resolution {
            wheel: None,
            continuation: None,
            chosen_creature_type: None,
            chosen_color: None,
            explore: None,
            evidence: None,
            item,
            program,
            pc: 0,
            target_group: 0,
            moved_count: None,
            connive: None,
            mana_payment: None,
            forage: None,
            discards: [0; 2],
            selected_discards: [Vec::new(), Vec::new()],
            milled: Vec::new(),
            milled_selection: None,
            library: None,
            selection: None,
            target_library: None,
            search: None,
            optional_searches: None,
            choice: None,
        });
        g.run_program();
        assert_eq!(g.observe(Player::P0).library_choice.is_some(), condition);
        if condition {
            assert!(g.observe(Player::P1).library_choice.is_none());
            arrange(&mut g, LibraryDestination::Top);
        }
        assert_eq!(g.state.players[0].life, if condition { 22 } else { 24 });
        assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    }
}
