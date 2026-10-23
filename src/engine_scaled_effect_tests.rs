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

#[test]
fn rockslide_reads_later_lands_and_counts_nonbasic_owned_lands_only() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    named(&mut g, Player::P0, "Temple Garden", Zone::Battlefield);
    named(&mut g, Player::P1, "Forest", Zone::Battlefield);
    named(&mut g, Player::P0, "Forest", Zone::Graveyard);
    let victim = named(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
    let spell = funded(&mut g, "Rumbling Rockslide");
    cast(&mut g, spell, Some(Target::Card(victim)));
    // Response-time board changes must affect the eventual quantity.
    named(&mut g, Player::P0, "Mountain", Zone::Battlefield);
    passes(&mut g);
    assert_eq!(g.object_view(victim).damage, 3);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
}

#[test]
fn rockslide_zero_and_generation_invalid_target_deal_no_damage() {
    for reenter in [false, true] {
        let mut g = scenario();
        let victim = named(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
        let spell = funded(&mut g, "Rumbling Rockslide");
        cast(&mut g, spell, Some(Target::Card(victim)));
        if reenter {
            named(&mut g, Player::P0, "Mountain", Zone::Battlefield);
            g.move_object(victim, Zone::Hand);
            g.move_object(victim, Zone::Battlefield);
        }
        passes(&mut g);
        assert_eq!(g.object_view(victim).damage, 0);
        assert!(!g.object_view(victim).damaged_this_turn);
    }
}

#[test]
fn slash_sums_owned_creatures_and_equipment_as_separate_counts() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    named(&mut g, Player::P0, "Goldvein Pick", Zone::Battlefield);
    named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    named(&mut g, Player::P1, "Goldvein Pick", Zone::Battlefield);
    let victim = named(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
    let spell = funded(&mut g, "Slash of Light");
    cast(&mut g, spell, Some(Target::Card(victim)));
    passes(&mut g);
    assert_eq!(g.object_view(victim).damage, 2);
}

#[test]
fn volley_includes_source_changeling_and_goblin_tokens_but_rejects_friendly_targets() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Three Tree Mascot", Zone::Battlefield);
    let token = Card::catalogue()
        .find(|c| c.is_token() && c.has_subtype("Goblin"))
        .unwrap();
    g.create_tokens(Player::P0, token, 1, false);
    named(&mut g, Player::P1, "Goblin Piker", Zone::Battlefield);
    let victim = named(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
    let v = enter(&mut g, "Volley Veteran");
    g.prepare_triggers();
    illegal(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(v),
        },
    );
    trigger(&mut g, &[Target::Card(victim)]);
    passes(&mut g);
    assert_eq!(g.object_view(victim).damage, 3);
}

#[test]
fn cat_gator_lifelink_uses_actual_swamp_subtypes_after_source_departure() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Swamp", Zone::Battlefield);
    named(&mut g, Player::P0, "Overgrown Tomb", Zone::Battlefield);
    named(&mut g, Player::P1, "Swamp", Zone::Battlefield);
    let source = enter(&mut g, "Cat-Gator");
    trigger(&mut g, &[Target::Player(Player::P1)]);
    g.move_object(source, Zone::Hand);
    named(&mut g, Player::P0, "Swamp", Zone::Battlefield);
    passes(&mut g);
    assert_eq!(g.state.players[1].life, 17);
    assert_eq!(g.state.players[0].life, 23);
    assert_eq!(g.state.players[0].life_gained_this_turn, 3);
    assert_eq!(g.state.players[0].life_gain_events_this_turn, 1);
}

#[test]
fn cat_gator_zero_damage_neither_gains_life_nor_triggers_pridemate() {
    let mut g = scenario();
    let pride = named(&mut g, Player::P0, "Ajani's Pridemate", Zone::Battlefield);
    enter(&mut g, "Cat-Gator");
    trigger(&mut g, &[Target::Player(Player::P1)]);
    passes(&mut g);
    assert_eq!(g.state.players[0].life_gain_events_this_turn, 0);
    assert_eq!(g.state.players[0].life_gained_this_turn, 0);
    assert_eq!(g.object_view(pride).counters.plus_one, 0);
    assert!(g.state.stack.is_empty());
}

#[test]
fn archway_angel_multiplies_live_gate_count_and_generates_one_gain_event() {
    let mut g = scenario();
    let pride = named(&mut g, Player::P0, "Ajani's Pridemate", Zone::Battlefield);
    named(&mut g, Player::P0, "Azorius Guildgate", Zone::Battlefield);
    named(&mut g, Player::P1, "Azorius Guildgate", Zone::Battlefield);
    let angel = enter(&mut g, "Archway Angel");
    trigger(&mut g, &[]);
    g.move_object(angel, Zone::Hand);
    named(&mut g, Player::P0, "Boros Guildgate", Zone::Battlefield);
    passes(&mut g);
    assert_eq!(g.state.players[0].life, 24);
    assert_eq!(g.state.players[0].life_gained_this_turn, 4);
    assert_eq!(g.state.players[0].life_gain_events_this_turn, 1);
    trigger(&mut g, &[]);
    passes(&mut g);
    assert_eq!(g.object_view(pride).counters.plus_one, 1);
}

#[test]
fn archway_life_overflow_restores_entire_unresolved_trigger_and_revision() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Azorius Guildgate", Zone::Battlefield);
    enter(&mut g, "Archway Angel");
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
    assert_eq!(g.state.players[0].life_gained_this_turn, 0);
}

#[test]
fn coordinated_modes_keep_separate_targets_and_route_damage_to_loyalty() {
    for mode in [0, 1] {
        let mut g = scenario();
        named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let walker = named(&mut g, Player::P1, "Ajani, Caller of the Pride", Zone::Hand);
        g.move_object(walker, Zone::Battlefield);
        let enchant = named(&mut g, Player::P1, "Clachan Festival", Zone::Battlefield);
        let spell = funded(&mut g, "Coordinated Maneuver");
        replay(&mut g, Action::AnnounceCast { card: spell });
        replay(&mut g, Action::ChooseMode { mode });
        let wrong = if mode == 0 { enchant } else { walker };
        illegal(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(wrong),
            },
        );
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(if mode == 0 { walker } else { enchant }),
            },
        );
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, pay);
        passes(&mut g);
        assert_eq!(
            g.object_view(walker).counters.loyalty,
            if mode == 0 { 2 } else { 4 }
        );
        assert_eq!(
            g.object_view(enchant).zone,
            if mode == 0 {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            }
        );
    }
}

#[test]
fn jet_both_hybrid_creature_bodies_are_live_and_death_targets_are_distinct() {
    let mut g = scenario();
    let bear = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let dragon = named(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
    let jet = enter(&mut g, "Jet, Freedom Fighter");
    trigger(&mut g, &[Target::Card(dragon)]);
    passes(&mut g);
    assert_eq!(g.object_view(dragon).damage, 2);
    g.move_object(jet, Zone::Graveyard);
    g.prepare_triggers();
    trigger(&mut g, &[Target::Card(bear), Target::Card(dragon)]);
    passes(&mut g);
    assert_eq!(g.object_view(bear).counters.plus_one, 1);
    assert_eq!(g.object_view(dragon).counters.plus_one, 1);
}

#[test]
fn smaug_upkeep_treasure_and_attack_quantity_are_complete() {
    let mut g = scenario();
    let smaug = named(
        &mut g,
        Player::P0,
        "Smaug the Magnificent",
        Zone::Battlefield,
    );
    g.begin_turn();
    g.give_priority();
    trigger(&mut g, &[]);
    passes(&mut g);
    assert_eq!(
        g.state
            .objects
            .iter()
            .filter(|o| o.zone == Zone::Battlefield
                && o.card.is_token()
                && o.card.has_subtype("Treasure"))
            .count(),
        1
    );
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    replay(&mut g, Action::Attack { card: smaug });
    replay(&mut g, Action::FinishAttackers);
    trigger(&mut g, &[Target::Player(Player::P1)]);
    passes(&mut g);
    assert_eq!(g.state.players[1].life, 19);
}

#[test]
fn scaled_amount_ir_rejects_unrepresentable_damage_gain_and_multiplication_atomically() {
    // Synthetic IR covers generic numeric boundaries without admitting a fake card.
    let cases: &[(u128, &'static [Instruction])] = &[
        (
            i128::MAX as u128 + 1,
            &[
                Instruction::May { skip: 1 },
                Instruction::DamageScaled(AmountValue::LifeGainedThisTurn),
            ],
        ),
        (
            i128::MAX as u128 + 1,
            &[
                Instruction::May { skip: 1 },
                Instruction::GainLifeScaled(PlayerSet::Controller, AmountValue::LifeGainedThisTurn),
            ],
        ),
        (
            u128::MAX,
            &[
                Instruction::May { skip: 1 },
                Instruction::GainLifeScaled(
                    PlayerSet::Controller,
                    AmountValue::Multiple {
                        value: &AmountValue::LifeGainedThisTurn,
                        factor: 2,
                    },
                ),
            ],
        ),
    ];
    for &(amount, program) in cases {
        let mut g = scenario();
        g.state.players[0].life_gained_this_turn = amount;
        let spell = funded(&mut g, "Shock");
        let mut item = g.cast_item(Player::P0, spell, [0; 6]);
        item.targets.push(BoundTarget {
            target: Target::Player(Player::P1),
            object: None,
            group: 0,
        });
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
            legendary_search: None,
            optional_searches: None,
            choice: None,
        });
        g.run_program();
        let before = g.snapshot();
        let rev = g.revision();
        let views = [
            serde_json::to_value(g.observe(Player::P0)).unwrap(),
            serde_json::to_value(g.observe(Player::P1)).unwrap(),
        ];
        for _ in 0..2 {
            assert!(matches!(
                g.apply(g.actor().unwrap(), Action::ChooseOptional { accept: true }),
                Err(Error::CapacityExceeded(_))
            ));
            assert_eq!(g.snapshot(), before);
            assert_eq!(g.revision(), rev);
            assert_eq!(
                serde_json::to_value(g.observe(Player::P0)).unwrap(),
                views[0]
            );
            assert_eq!(
                serde_json::to_value(g.observe(Player::P1)).unwrap(),
                views[1]
            );
        }
        replay(&mut g, Action::ChooseOptional { accept: false });
        assert_eq!(g.state.players[1].life, 20);
    }
}

#[test]
fn cat_gator_lifelink_capacity_restores_damage_and_both_players() {
    let mut g = scenario();
    named(&mut g, Player::P0, "Swamp", Zone::Battlefield);
    enter(&mut g, "Cat-Gator");
    trigger(&mut g, &[Target::Player(Player::P1)]);
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
    assert_eq!(g.state.players[1].life, 20);
}

#[test]
fn zero_gate_gain_does_not_create_a_life_event() {
    let mut g = scenario();
    let pride = named(&mut g, Player::P0, "Ajani's Pridemate", Zone::Battlefield);
    enter(&mut g, "Archway Angel");
    trigger(&mut g, &[]);
    passes(&mut g);
    assert_eq!(g.state.players[0].life, 20);
    assert_eq!(g.state.players[0].life_gained_this_turn, 0);
    assert_eq!(g.state.players[0].life_gain_events_this_turn, 0);
    assert_eq!(g.object_view(pride).counters.plus_one, 0);
    assert!(g.state.stack.is_empty());
}
