use super::tests::{act, scenario, spawn};
use super::*;
use crate::cards::CastFilter;
const NAMES: [&str; 13] = [
    "Angry Rabble",
    "Brineborn Cutthroat",
    "Enraged Flamecaster",
    "Kulrath Mystic",
    "Lurking Lizards",
    "Mage Tower Referee",
    "Nightmare Sower",
    "Skybeast Tracker",
    "Solarium Sentry",
    "Spider Manifestation",
    "Stormkeld Prowler",
    "Tanufel Rimespeaker",
    "Unwelcome Sprite",
];
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    spawn(g, p, c, z)
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [20; 6];
    }
    g
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let e = g.snapshot();
    assert_eq!(f.snapshot(), e);
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, r, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), e);
}
fn choose(g: &mut Game, target: Option<Target>) {
    let actions = g.legal_actions();
    let a = target
        .and_then(|t| {
            actions
                .iter()
                .find(|a| **a == Action::ChooseTarget { target: t })
        })
        .or_else(|| {
            actions.iter().find(|a| {
                matches!(
                    a,
                    Action::ChooseSpellCost { .. }
                        | Action::PayCosts { .. }
                        | Action::PutTrigger { .. }
                        | Action::Pass
                        | Action::FinishTargets
                        | Action::ArrangeLibrary { .. }
                        | Action::Discard { .. }
                )
            })
        })
        .unwrap_or_else(|| {
            panic!(
                "fixture decision {}",
                g.observe(g.actor().unwrap()).decision
            )
        })
        .clone();
    replay(g, a);
}
fn settle(g: &mut Game, target: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..160 {
        if g.outcome().is_some() || g.state.pending == Pending::Priority && g.state.stack.is_empty()
        {
            return;
        }
        choose(g, target);
    }
    panic!("unsettled");
}
fn announce(g: &mut Game, id: CardId, target: Option<Target>) {
    replay(g, Action::AnnounceCast { card: id });
    while g
        .state
        .announcing
        .as_ref()
        .is_some_and(|a| a.item.kind == StackKind::Spell)
    {
        choose(g, target);
    }
}
fn cast(g: &mut Game, id: CardId, target: Option<Target>) {
    announce(g, id, target);
    settle(g, target);
}
#[test]
fn all_thirteen_complete_bodies_cast_and_resolve_real_qualified_events_for_both_seats() {
    for p in [Player::P0, Player::P1] {
        for (i, n) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let source = named(&mut g, p, n, Zone::Hand);
            cast(&mut g, source, None);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let before = g.object_view(source);
            let life = g.state.players[p.index()].life;
            let lib = g.state.players[p.index()].library.len();
            if i == 9 {
                g.state.objects[source as usize].tapped = true;
            }
            let caster = if i == 8 { p.other() } else { p };
            g.state.active = if [1, 6, 12].contains(&i) {
                p.other()
            } else {
                caster
            };
            g.state.priority = caster;
            let trigger = named(
                &mut g,
                caster,
                if i == 5 {
                    "Warleader's Call"
                } else if [1, 6, 8, 12].contains(&i) {
                    "Opt"
                } else {
                    "Shivan Dragon"
                },
                Zone::Hand,
            );
            announce(&mut g, trigger, None);
            assert!(
                g.state
                    .stack
                    .iter()
                    .chain(&g.state.pending_triggers)
                    .chain(&g.state.ordering_triggers)
                    .chain(g.state.announcing.iter().map(|a| &a.item))
                    .any(|s| s.card == source),
                "{n}"
            );
            settle(&mut g, Some(Target::Card(enemy)));
            let after = g.object_view(source);
            match i {
                0 => assert_eq!(g.state.players[p.other().index()].life, 19),
                1 | 4 => assert_eq!(after.counters.plus_one, before.counters.plus_one + 1),
                2 => assert_eq!(g.state.players[p.other().index()].life, 18),
                3 => {
                    assert_eq!(after.power, before.power + 2);
                    assert!(after.keywords.contains(Keywords::VIGILANCE));
                }
                5 => assert_eq!(after.counters.plus_one, before.counters.plus_one + 1),
                6 => assert_eq!(g.object_view(enemy).counters.minus_one, 1),
                7 => assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
                    && o.owner == p
                    && o.card.token_info().is_some_and(|t| t.subtypes == "Food"))),
                8 => assert_eq!(g.state.players[p.index()].life, life + 2),
                9 => assert!(!after.tapped),
                10 => assert_eq!(after.counters.plus_one, before.counters.plus_one + 2),
                11 => assert_eq!(g.state.players[p.index()].library.len(), lib - 1),
                12 => assert!(g.state.players[p.index()].library.len() < lib),
                _ => unreachable!(),
            }
        }
    }
}
#[test]
fn printed_mana_value_boundaries_and_relations_are_exact() {
    for p in [Player::P0, Player::P1] {
        for (n, positive, negative) in [
            ("Tanufel Rimespeaker", "Ravenous Giant", "Divining Duelist"),
            ("Stormkeld Prowler", "Serra Redeemer", "Ravenous Giant"),
            ("Solarium Sentry", "Bear Cub", "Divining Duelist"),
        ] {
            for owner in [p, p.other()] {
                for (spell, qualifies) in [(positive, true), (negative, false)] {
                    let mut g = setup(p);
                    let source = named(&mut g, p, n, Zone::Battlefield);
                    g.record_spell_cast(owner, Card::from_name(spell).unwrap());
                    let related = if n == "Solarium Sentry" {
                        owner != p
                    } else {
                        owner == p
                    };
                    assert_eq!(
                        g.state
                            .pending_triggers
                            .iter()
                            .filter(|s| s.card == source)
                            .count(),
                        usize::from(qualifies && related),
                        "{n} {spell} {owner:?}"
                    );
                }
            }
        }
    }
}
#[test]
fn multicolored_is_printed_color_not_paid_color_or_color_identity() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Mage Tower Referee", Zone::Battlefield);
    for (name, want) in [
        ("Bear Cub", false),
        ("Goldvein Pick", false),
        ("EPF Point Squad", true),
        ("Warleader's Call", true),
    ] {
        g.state.pending_triggers.clear();
        g.record_spell_cast(Player::P0, Card::from_name(name).unwrap());
        assert_eq!(
            g.state
                .pending_triggers
                .iter()
                .filter(|s| s.card == source)
                .count(),
            usize::from(want),
            "{name}"
        );
    }
}
#[test]
fn opponent_turn_gate_is_event_only_and_does_not_recheck_at_resolution() {
    for p in [Player::P0, Player::P1] {
        for n in [NAMES[1], NAMES[6], NAMES[12]] {
            let mut g = setup(p);
            let source = named(&mut g, p, n, Zone::Battlefield);
            g.record_spell_cast(p, Card::from_name("Opt").unwrap());
            assert!(g.state.pending_triggers.is_empty());
            g.state.active = p.other();
            g.record_spell_cast(p.other(), Card::from_name("Opt").unwrap());
            assert!(g.state.pending_triggers.is_empty());
            g.record_spell_cast(p, Card::from_name("Opt").unwrap());
            assert_eq!(g.state.pending_triggers[0].card, source);
            g.state.active = p;
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            settle(&mut g, Some(Target::Card(enemy)));
            match n {
                n if n == NAMES[1] => assert_eq!(g.object_view(source).counters.plus_one, 1),
                n if n == NAMES[6] => assert_eq!(g.object_view(enemy).counters.minus_one, 1),
                _ => assert!(g.state.stack.is_empty()),
            }
        }
    }
}
#[test]
fn two_brid_printed_six_triggers_after_three_mana_payment_and_preserves_cast_count() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Stormkeld Prowler", Zone::Battlefield);
    let target = named(&mut g, Player::P1, "Shivan Dragon", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Defibrillating Current", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: spell });
    let cost = g
        .observe(Player::P0)
        .cost_choices
        .into_iter()
        .min_by_key(|c| c.cost.generic + c.cost.colored.iter().sum::<u32>())
        .unwrap();
    assert_eq!(cost.cost.generic + cost.cost.colored.iter().sum::<u32>(), 3);
    replay(&mut g, Action::ChooseSpellCost { cost: cost.index });
    while g
        .state
        .announcing
        .as_ref()
        .is_some_and(|a| a.item.kind == StackKind::Spell)
    {
        choose(&mut g, Some(Target::Card(target)));
    }
    assert_eq!(g.state.players[0].mana.iter().sum::<u32>(), 117);
    settle(&mut g, None);
    assert_eq!(g.object_view(source).counters.plus_one, 2);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
}
#[test]
fn announcement_and_payment_proposals_do_not_trigger_and_illegal_actions_are_atomic() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Tanufel Rimespeaker", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Shivan Dragon", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: spell });
    assert!(g.state.pending_triggers.is_empty());
    assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
    let s = g.snapshot();
    let r = g.revision();
    assert!(g.apply(Player::P1, Action::Pass).is_err());
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
    choose(&mut g, None);
    assert!(
        g.state
            .stack
            .iter()
            .chain(&g.state.pending_triggers)
            .any(|s| s.card == source)
    );
}
#[test]
fn source_departure_and_reentry_cannot_redirect_old_counter_or_untap() {
    for n in ["Stormkeld Prowler", "Spider Manifestation"] {
        let mut g = setup(Player::P0);
        let source = named(&mut g, Player::P0, n, Zone::Battlefield);
        g.record_spell_cast(Player::P0, Card::from_name("Shivan Dragon").unwrap());
        g.move_object(source, Zone::Exile);
        g.move_object(source, Zone::Battlefield);
        g.state.objects[source as usize].tapped = true;
        settle(&mut g, None);
        assert_eq!(g.object_view(source).counters.plus_one, 0);
        assert!(g.object_view(source).tapped);
    }
}
#[test]
fn captured_damage_and_draw_survive_source_departure_and_private_surveil_is_owned() {
    for n in [
        "Enraged Flamecaster",
        "Tanufel Rimespeaker",
        "Unwelcome Sprite",
    ] {
        let mut g = setup(Player::P0);
        let source = named(&mut g, Player::P0, n, Zone::Battlefield);
        g.state.active = Player::P1;
        g.record_spell_cast(Player::P0, Card::from_name("Shivan Dragon").unwrap());
        g.move_object(source, Zone::Exile);
        g.prepare_triggers();
        let lib = g.state.players[0].library.len();
        for _ in 0..50 {
            if g.observe(g.actor().unwrap()).library_choice.is_some() {
                assert!(g.observe(Player::P1).library_choice.is_none());
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::ArrangeLibrary { .. }))
                    .unwrap();
                replay(&mut g, a);
                break;
            }
            if g.state.stack.is_empty() && g.state.pending == Pending::Priority {
                break;
            }
            choose(&mut g, None);
        }
        settle(&mut g, None);
        if n == "Enraged Flamecaster" {
            assert_eq!(g.state.players[1].life, 18);
        }
        if n == "Tanufel Rimespeaker" {
            assert_eq!(g.state.players[0].library.len(), lib - 1);
        }
    }
}
#[test]
fn source_counter_and_completed_cast_overflows_restore_entire_state() {
    for stage in [0, 1] {
        let mut g = setup(Player::P0);
        let source = named(&mut g, Player::P0, "Stormkeld Prowler", Zone::Battlefield);
        let spell = named(&mut g, Player::P0, "Shivan Dragon", Zone::Hand);
        if stage == 0 {
            g.state.players[0].spells_cast_this_turn = u64::MAX;
            replay(&mut g, Action::AnnounceCast { card: spell });
        } else {
            g.state.objects[source as usize].counters.plus_one = u64::MAX;
            announce(&mut g, spell, None);
            replay(&mut g, Action::Pass);
        }
        let action = if stage == 0 {
            g.legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap()
        } else {
            Action::Pass
        };
        let s = g.snapshot();
        let r = g.revision();
        assert!(matches!(
            g.apply(g.actor().unwrap(), action),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), s);
        assert_eq!(g.revision(), r);
    }
}
#[test]
fn opponent_turn_wrapper_preserves_once_and_intervening_metadata() {
    const EVENT: TriggerCondition = TriggerCondition::SpellCast {
        relation: CastRelation::You,
        filter: CastFilter::Any,
        ordinal: None,
    };
    const GUARD: TriggerCondition = TriggerCondition::InterveningIf {
        guard: StepGuard::GainedLife,
        event: &EVENT,
    };
    let c = TriggerCondition::OncePerTurn(&TriggerCondition::DuringOpponentsTurn(&GUARD));
    assert_eq!(c.event(), EVENT);
    assert!(c.once_per_turn());
    assert!(c.requires_opponents_turn());
    let mut g = setup(Player::P0);
    g.state.active = Player::P1;
    assert!(!g.trigger_guards_hold(
        c,
        Player::P0,
        ObjectRef {
            card: 0,
            generation: 0
        },
        false
    ));
    g.state.players[0].life_gained_this_turn = 1;
    assert!(g.trigger_guards_hold(
        c,
        Player::P0,
        ObjectRef {
            card: 0,
            generation: 0
        },
        false
    ));
}
#[test]
fn angry_rabble_activation_has_full_sorcery_cost_and_food_keeps_its_recipe() {
    let mut g = setup(Player::P0);
    let rabble = named(&mut g, Player::P0, "Angry Rabble", Zone::Hand);
    cast(&mut g, rabble, None);
    let a = Action::AnnounceActivate {
        card: rabble,
        ability: 0,
        counter_payment: Counters::default(),
    };
    g.state.active = Player::P1;
    let s = g.snapshot();
    assert!(g.apply(Player::P0, a.clone()).is_err());
    assert_eq!(g.snapshot(), s);
    g.state.active = Player::P0;
    replay(&mut g, a);
    while g.state.announcing.is_some() {
        choose(&mut g, None);
    }
    settle(&mut g, None);
    assert_eq!(g.object_view(rabble).counters.plus_one, 2);
    let tracker = named(&mut g, Player::P0, "Skybeast Tracker", Zone::Battlefield);
    g.record_spell_cast(Player::P0, Card::from_name("Shivan Dragon").unwrap());
    settle(&mut g, None);
    let food = g
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, o)| {
            o.zone == Zone::Battlefield && o.card.token_info().is_some_and(|t| t.subtypes == "Food")
        })
        .unwrap()
        .0 as CardId;
    let life = g.state.players[0].life;
    replay(
        &mut g,
        Action::AnnounceActivate {
            card: food,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    while g.state.announcing.is_some() {
        choose(&mut g, None);
    }
    settle(&mut g, None);
    assert_eq!(g.state.players[0].life, life + 3);
    assert_eq!(g.object_view(food).zone, Zone::Outside);
    assert_eq!(g.object_view(tracker).zone, Zone::Battlefield);
}
#[test]
fn spider_mana_routes_and_funding_queries_never_borrow_a_future_cast_untap() {
    for color in [3, 4] {
        let mut g = setup(Player::P0);
        let source = named(&mut g, Player::P0, "Spider Manifestation", Zone::Hand);
        cast(&mut g, source, None);
        g.state.objects[source as usize].summoning_sick = false;
        g.state.players[0].mana = [0; 6];
        let saved = g.snapshot();
        let revision = g.revision();
        let mut required = [0; 6];
        required[color] = 1;
        assert!(g.can_fund(super::funding::FundingGoal {
            actor: Player::P0,
            required,
            generic: 0,
            reserve: None,
            source: None
        }));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), revision);
        let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,production,..}if *card==source&&production[color]==1)).unwrap();
        replay(&mut g, a);
        assert_eq!(g.state.players[0].mana[color], 1);
        assert!(g.object_view(source).tapped);
        let spell = named(&mut g, Player::P0, "Shivan Dragon", Zone::Hand);
        let saved = g.snapshot();
        assert!(
            g.apply(Player::P0, Action::AnnounceCast { card: spell })
                .is_err()
        );
        assert_eq!(g.snapshot(), saved);
    }
}
#[test]
fn a_countered_spell_still_caused_one_completed_cast_event() {
    let mut g = setup(Player::P0);
    let observer = named(&mut g, Player::P0, "Stormkeld Prowler", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Shivan Dragon", Zone::Hand);
    announce(&mut g, spell, None);
    replay(&mut g, Action::Pass);
    replay(&mut g, Action::Pass);
    assert_eq!(g.object_view(observer).counters.plus_one, 2);
    assert_eq!(g.object_view(spell).zone, Zone::Stack);
    replay(&mut g, Action::Pass);
    assert_eq!(g.actor(), Some(Player::P1));
    let cancel = named(&mut g, Player::P1, "Cancel", Zone::Hand);
    announce(&mut g, cancel, Some(Target::Spell(spell)));
    settle(&mut g, None);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(g.object_view(observer).counters.plus_one, 2);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
    assert_eq!(g.state.players[1].spells_cast_this_turn, 1);
}
#[test]
fn nightmare_optional_targets_and_reentered_objects_do_not_receive_a_stale_counter() {
    for select in [false, true] {
        let mut g = setup(Player::P0);
        named(&mut g, Player::P0, "Nightmare Sower", Zone::Battlefield);
        let bear = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        g.state.active = Player::P1;
        g.record_spell_cast(Player::P0, Card::from_name("Opt").unwrap());
        g.prepare_triggers();
        replay(
            &mut g,
            if select {
                Action::ChooseTarget {
                    target: Target::Card(bear),
                }
            } else {
                Action::FinishTargets
            },
        );
        if select {
            g.move_object(bear, Zone::Hand);
            g.move_object(bear, Zone::Battlefield);
        }
        settle(&mut g, None);
        assert_eq!(g.object_view(bear).counters.minus_one, 0);
    }
}
