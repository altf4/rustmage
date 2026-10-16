use super::tests::{act, scenario, spawn};
use super::*;
use crate::cards::{CastFilter, CastTarget};
const NAMES: [&str; 17] = [
    "Colleen Wing, Street Samurai",
    "Danitha, Spear of Agony",
    "Danitha, Sword of Hope",
    "Firespitter Whelp",
    "Forum Necroscribe",
    "Gnarlback Rhino",
    "Graduation Day",
    "Informed Inkwright",
    "Inkling Mascot",
    "Inkshape Demonstrator",
    "Lecturing Scornmage",
    "Madame Hydra",
    "Melancholic Poet",
    "Mockingbird, Ace Agent",
    "Rehearsed Debater",
    "Rodeo Pyromancers",
    "Stirring Hopesinger",
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
        s.mana = [30; 6];
    }
    g
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let p = g.actor().unwrap();
    let r = g.revision();
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
                        | Action::ChooseOptional { accept: true }
                )
            })
        })
        .or_else(|| {
            actions
                .iter()
                .find(|a| matches!(a, Action::ChooseTarget { .. }))
        })
        .expect("fixture continuation")
        .clone();
    if g.observe(g.actor().unwrap()).library_choice.is_some() {
        assert!(
            g.observe(g.actor().unwrap().other())
                .library_choice
                .is_none()
        );
    }
    replay(g, a);
}
fn settle(g: &mut Game, target: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..200 {
        if g.outcome().is_some()
            || g.state.stack.is_empty()
                && g.state.announcing.is_none()
                && g.state.pending == Pending::Priority
        {
            return;
        }
        choose(g, target);
    }
    panic!("unsettled")
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
fn seventeen_complete_bodies_cast_for_both_players_and_execute_real_events() {
    for p in [Player::P0, Player::P1] {
        for (i, n) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let source = named(&mut g, p, n, Zone::Hand);
            cast(&mut g, source, None);
            let mate = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let dead = named(&mut g, p, "Bear Cub", Zone::Graveyard);
            let old = g.object_view(source);
            let lib = g.state.players[p.index()].library.len();
            let life = g.state.players[p.index()].life;
            let red_before = g.state.players[p.index()].mana[3];
            g.state.players[p.index()].spells_cast_this_turn = 0;
            let (spell, target) = match i {
                1 => ("Lightning Bolt", Some(Target::Card(enemy))),
                2 => ("Goldvein Pick", None),
                3 | 15 => ("Opt", None),
                11 => ("HYDRA Assault Robot", None),
                5 => ("Giant Growth", Some(Target::Card(source))),
                _ => ("Giant Growth", Some(Target::Card(mate))),
            };
            let id = named(&mut g, p, spell, Zone::Hand);
            announce(&mut g, id, target);
            settle(
                &mut g,
                if i == 4 {
                    Some(Target::Card(dead))
                } else {
                    Some(Target::Card(mate))
                },
            );
            let now = g.object_view(source);
            match i {
                0 | 1 | 10 | 13 => {
                    assert_eq!(now.counters.plus_one, old.counters.plus_one + 1, "{n}")
                }
                2 | 5 => assert_eq!(g.state.players[p.index()].library.len(), lib - 1),
                3 => assert_eq!(g.state.players[p.other().index()].life, 19),
                4 => assert_eq!(g.object_view(dead).zone, Zone::Battlefield),
                6 => assert_eq!(g.object_view(mate).counters.plus_one, 1),
                7 | 11 => {
                    assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
                        && o.card.token_info().is_some_and(
                            |t| t.subtypes == if i == 7 { "Inkling" } else { "Villain" }
                        )))
                }
                8 => assert!(now.keywords.contains(Keywords::FLYING)),
                9 => {
                    assert_eq!(now.power, old.power + 1);
                    assert!(now.keywords.contains(Keywords::LIFELINK));
                }
                12 => {
                    assert_eq!(g.state.players[p.index()].life, life + 1);
                    assert_eq!(g.state.players[p.other().index()].life, 19);
                }
                14 => assert_eq!(
                    (now.power, now.toughness),
                    (old.power + 1, old.toughness + 1)
                ),
                15 => assert_eq!(g.state.players[p.index()].mana[3], red_before + 2),
                16 => {
                    assert_eq!(now.counters.plus_one, 1);
                    assert_eq!(g.object_view(mate).counters.plus_one, 1);
                }
                _ => unreachable!(),
            }
        }
    }
}
#[test]
fn bound_target_filters_distinguish_relations_players_spells_source_and_current_incarnation() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Gnarlback Rhino", Zone::Battlefield);
    let mate = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let enemy = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let land = named(&mut g, Player::P0, "Forest", Zone::Battlefield);
    let view = g.object_view(source);
    let card = Card::GiantGrowth;
    for (target, you, any, opp, self_) in [
        (Target::Card(source), true, true, false, true),
        (Target::Card(mate), true, true, false, false),
        (Target::Card(enemy), false, true, true, false),
        (Target::Card(land), false, false, false, false),
        (Target::Player(Player::P0), false, false, false, false),
        (Target::Player(Player::P1), false, false, true, false),
    ] {
        let bound = g.bind_target(target);
        for (req, want) in [
            (CastTarget::Creature(CastRelation::You), you),
            (CastTarget::Creature(CastRelation::Any), any),
            (CastTarget::OpponentOrCreature, opp),
            (CastTarget::Source, self_),
        ] {
            assert_eq!(
                g.cast_filter_matches(
                    CastFilter::Targeted {
                        filter: &CastFilter::Any,
                        target: req
                    },
                    card,
                    &view,
                    &[bound]
                ),
                want
            );
        }
    }
    let enemy_bound = g.bind_target(Target::Card(enemy));
    g.state.objects[enemy as usize].gained_keywords = Keywords::HEXPROOF;
    assert!(!g.target_matches(
        TargetRule::Creature,
        Player::P0,
        view.reference,
        Target::Card(enemy)
    ));
    assert!(g.cast_filter_matches(
        CastFilter::Targeted {
            filter: &CastFilter::Any,
            target: CastTarget::Creature(CastRelation::Any)
        },
        card,
        &view,
        &[enemy_bound]
    ));
    let old_source_bound = g.bind_target(Target::Card(source));
    g.move_object(source, Zone::Exile);
    g.move_object(source, Zone::Battlefield);
    let fresh_source = g.object_view(source);
    assert!(!g.cast_filter_matches(
        CastFilter::Targeted {
            filter: &CastFilter::Any,
            target: CastTarget::Source
        },
        card,
        &fresh_source,
        &[old_source_bound]
    ));
    let bound = g.bind_target(Target::Card(mate));
    g.move_object(mate, Zone::Exile);
    g.move_object(mate, Zone::Battlefield);
    assert!(!g.cast_filter_matches(
        CastFilter::Targeted {
            filter: &CastFilter::Any,
            target: CastTarget::Creature(CastRelation::Any)
        },
        card,
        &view,
        &[bound]
    ));
}
#[test]
fn selecting_a_target_does_not_fire_before_costs_and_a_sacrificed_target_does_not_qualify() {
    for observer in [
        "Colleen Wing, Street Samurai",
        "Lecturing Scornmage",
        "Danitha, Sword of Hope",
    ] {
        let mut g = setup(Player::P0);
        let source = named(&mut g, Player::P0, observer, Zone::Battlefield);
        let mate = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
        let spell = named(&mut g, Player::P0, "Final Vengeance", Zone::Hand);
        replay(&mut g, Action::AnnounceCast { card: spell });
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(mate),
            },
        );
        if matches!(g.state.pending, Pending::Targets { .. }) {
            replay(&mut g, Action::FinishTargets);
        }
        assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
        assert!(g.state.pending_triggers.iter().all(|s| s.card != source));
        let object = g.object_ref(mate);
        replay(&mut g, Action::ChooseAdditionalCost { object });
        assert_eq!(g.object_view(mate).zone, Zone::Battlefield);
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, payment);
        assert_eq!(g.object_view(mate).zone, Zone::Graveyard);
        assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
        assert!(g.state.stack.iter().all(|s| s.kind == StackKind::Spell));
        assert!(g.state.pending_triggers.is_empty());
        settle(&mut g, None);
        assert_eq!(g.object_view(source).counters.plus_one, 0);
    }
}
#[test]
fn multi_target_spell_qualifies_once_and_abilities_never_count_as_casts() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Lecturing Scornmage", Zone::Battlefield);
    let a = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let b = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let spell = named(&mut g, Player::P0, "Out Cold", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: spell });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(a),
        },
    );
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(b),
        },
    );
    while g.state.announcing.is_some() {
        choose(&mut g, None);
    }
    settle(&mut g, None);
    assert_eq!(g.object_view(source).counters.plus_one, 1);
    let guard = named(&mut g, Player::P0, "Frostbridge Guard", Zone::Battlefield);
    let item = g.activation_item(Player::P0, guard, 0);
    g.push_stack_item(item);
    assert_eq!(g.state.players[0].spells_cast_this_turn, 1);
    assert!(g.state.pending_triggers.is_empty());
}
#[test]
fn equipment_union_once_limit_resets_next_turn_and_opponent_cast_does_not_consume_it() {
    let mut g = setup(Player::P0);
    named(
        &mut g,
        Player::P0,
        "Danitha, Sword of Hope",
        Zone::Battlefield,
    );
    let mate = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let lib = g.state.players[0].library.len();
    let pick = named(&mut g, Player::P0, "Goldvein Pick", Zone::Hand);
    cast(&mut g, pick, None);
    assert_eq!(g.state.players[0].library.len(), lib - 1);
    let pump = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    cast(&mut g, pump, Some(Target::Card(mate)));
    assert_eq!(g.state.players[0].library.len(), lib - 1);
    g.state.active = Player::P0;
    g.begin_turn();
    g.state.step = Step::Main1;
    g.give_priority();
    g.state.players[0].mana = [30; 6];
    let pump = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    cast(&mut g, pump, Some(Target::Card(mate)));
    assert_eq!(g.state.players[0].library.len(), lib - 2);
}
#[test]
fn repartee_requires_instant_sorcery_but_allows_either_players_creature() {
    for aura in [false, true] {
        let mut g = setup(Player::P0);
        let s = named(&mut g, Player::P0, "Lecturing Scornmage", Zone::Battlefield);
        let e = named(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
        let id = named(
            &mut g,
            Player::P0,
            if aura {
                "Untamed Hunger"
            } else {
                "Giant Growth"
            },
            Zone::Hand,
        );
        cast(&mut g, id, Some(Target::Card(e)));
        assert_eq!(g.object_view(s).counters.plus_one, u64::from(!aura));
    }
}
#[test]
fn source_departure_after_completed_cast_does_not_cancel_private_draw_trigger() {
    let mut g = setup(Player::P0);
    let s = named(&mut g, Player::P0, "Gnarlback Rhino", Zone::Battlefield);
    let id = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    announce(&mut g, id, Some(Target::Card(s)));
    let lib = g.state.players[0].library.len();
    let old = g.object_ref(s);
    g.move_object(s, Zone::Exile);
    g.move_object(s, Zone::Battlefield);
    assert_ne!(old, g.object_ref(s));
    settle(&mut g, None);
    assert_eq!(g.state.players[0].library.len(), lib - 1);
    assert_eq!(g.object_view(s).power, 4);
}
#[test]
fn first_spell_ordinal_tracks_each_player_and_resets_only_with_turn() {
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, "Rodeo Pyromancers", Zone::Battlefield);
    for n in 0..2 {
        let id = named(&mut g, Player::P0, "Opt", Zone::Hand);
        cast(&mut g, id, None);
        assert_eq!(g.state.players[0].mana[3], 32);
        assert_eq!(g.state.players[0].spells_cast_this_turn, n + 1);
    }
    g.state.active = Player::P1;
    g.begin_turn();
    g.state.step = Step::Main1;
    g.state.priority = Player::P0;
    g.state.players[0].mana = [30; 6];
    let id = named(&mut g, Player::P0, "Opt", Zone::Hand);
    cast(&mut g, id, None);
    assert_eq!(g.state.players[0].mana[3], 32);
}
#[test]
fn completed_targeted_cast_capacity_failure_is_atomic() {
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, "Lecturing Scornmage", Zone::Battlefield);
    let mate = named(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let id = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    replay(&mut g, Action::AnnounceCast { card: id });
    replay(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(mate),
        },
    );
    if matches!(g.state.pending, Pending::Targets { .. }) {
        replay(&mut g, Action::FinishTargets);
    }
    g.state.next_stack_id = u64::MAX;
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    let s = g.snapshot();
    let r = g.revision();
    assert!(matches!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::CapacityExceeded(_))
    ));
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}

#[test]
fn targeted_cast_trigger_survives_an_actual_counterspell() {
    let mut g = setup(Player::P0);
    named(&mut g, Player::P0, "Gnarlback Rhino", Zone::Battlefield);
    let source = g.state.cast_sources[0];
    let spell = named(&mut g, Player::P0, "Giant Growth", Zone::Hand);
    announce(&mut g, spell, Some(Target::Card(source)));
    assert_eq!(g.state.stack.len(), 2);
    let counter = named(&mut g, Player::P1, "Counterspell", Zone::Hand);
    replay(&mut g, Action::Pass);
    announce(&mut g, counter, Some(Target::Spell(spell)));
    let lib = g.state.players[0].library.len();
    settle(&mut g, None);
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(g.object_view(source).power, 4);
    assert_eq!(g.state.players[0].library.len(), lib - 1);
}
#[test]
fn villain_and_noncreature_dragon_unions_keep_intrinsic_filters() {
    let mut g = setup(Player::P0);
    let source = named(&mut g, Player::P0, "Firespitter Whelp", Zone::Battlefield);
    let view = g.object_view(source);
    let filter = CastFilter::AnyOf(&[CastFilter::Noncreature, CastFilter::Subtype("Dragon")]);
    for (name, want) in [
        ("Bear Cub", false),
        ("Opt", true),
        ("Shivan Dragon", true),
        ("Three Tree Mascot", true),
    ] {
        assert_eq!(
            g.cast_filter_matches(filter, Card::from_name(name).unwrap(), &view, &[]),
            want
        );
    }
    let hydra = CastFilter::Subtype("Villain");
    assert!(g.cast_filter_matches(
        hydra,
        Card::from_name("HYDRA Assault Robot").unwrap(),
        &view,
        &[]
    ));
    assert!(g.cast_filter_matches(
        hydra,
        Card::from_name("Three Tree Mascot").unwrap(),
        &view,
        &[]
    ));
    assert!(!g.cast_filter_matches(hydra, Card::from_name("Bear Cub").unwrap(), &view, &[]));
}

#[test]
fn opponent_cast_targeting_owned_creature_or_source_never_counts_as_your_cast() {
    for name in [
        "Colleen Wing, Street Samurai",
        "Danitha, Sword of Hope",
        "Lecturing Scornmage",
        "Gnarlback Rhino",
    ] {
        let mut g = setup(Player::P1);
        let source = named(&mut g, Player::P0, name, Zone::Battlefield);
        let id = named(&mut g, Player::P1, "Giant Growth", Zone::Hand);
        let lib = g.state.players[0].library.len();
        cast(&mut g, id, Some(Target::Card(source)));
        assert_eq!(g.state.players[0].library.len(), lib);
        assert_eq!(g.object_view(source).counters.plus_one, 0);
        assert_eq!(g.state.players[0].spells_cast_this_turn, 0);
        assert_eq!(g.state.players[1].spells_cast_this_turn, 1);
        assert!(g.state.triggered_this_turn.is_empty());
    }
}
