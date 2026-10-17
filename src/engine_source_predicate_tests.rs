use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 13] = [
    "Akawalli, the Seething Tower",
    "Cephalid Inkmage",
    "Druid of the Spade",
    "First-Time Flyer",
    "Foggy Swamp Hunters",
    "Ghitu Lavarunner",
    "June, Bounty Hunter",
    "Kitesail Corsair",
    "Messenger Hawk",
    "Nightwhorl Hermit",
    "Skyward Spider",
    "Stoic Sphinx",
    "Twinblade Paladin",
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
fn pt(g: &Game, id: CardId) -> (i128, i128) {
    let o = g.object_view(id);
    (o.power, o.toughness)
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
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn settle(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..120 {
        if g.outcome().is_some() || g.state.stack.is_empty() && g.state.pending == Pending::Priority
        {
            return;
        }
        let p = g.actor().unwrap();
        if g.observe(p).search_choice.is_some()
            || g.observe(p).library_choice.is_some()
            || matches!(g.state.pending, Pending::EffectDiscard { .. })
        {
            let o = g.observe(p.other());
            assert!(
                o.search_choice.is_none() && o.library_choice.is_none() && o.actions.is_empty()
            );
        }
        let mut choices = g.legal_actions();
        choices.sort_by_key(|a| {
            !matches!(
                a,
                Action::ChooseSearch { .. }
                    | Action::ArrangeLibrary {
                        destination: LibraryDestination::Graveyard,
                        ..
                    }
            )
        });
        let a = choices
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::ChooseSearch { .. }
                        | Action::Discard { .. }
                        | Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishSearch
                        | Action::ArrangeLibrary { .. }
                )
            })
            .expect("fixture resolution");
        replay(g, a);
    }
    panic!("unsettled");
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(g, a);
}
fn full_cast(g: &mut Game, id: CardId) {
    replay(g, Action::AnnounceCast { card: id });
    if let Some(a) = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::ChooseSpellCost { .. }))
    {
        replay(g, a);
    }
    pay(g);
    settle(g);
}
fn grave(g: &mut Game, p: Player, n: usize, name: &str) -> Vec<CardId> {
    (0..n).map(|_| named(g, p, name, Zone::Graveyard)).collect()
}
fn token(g: &mut Game, p: Player, z: Zone) -> CardId {
    let c = Card::catalogue()
        .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Treasure"))
        .unwrap();
    spawn(g, p, c, z)
}
fn attack(g: &mut Game, ids: &[CardId]) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    for &id in ids {
        replay(g, Action::Attack { card: id });
    }
    replay(g, Action::FinishAttackers);
    settle(g);
    for _ in 0..40 {
        if g.state.pending == Pending::Blockers {
            return;
        }
        replay(g, Action::Pass);
    }
    panic!("no blockers step");
}
fn damage(g: &mut Game) {
    replay(g, Action::FinishBlockers);
    for _ in 0..80 {
        if g.state.step == Step::CombatDamage
            && g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
        {
            return;
        }
        if let Some(r) = g.damage_assignment_range() {
            replay(
                g,
                Action::AssignDamage {
                    target: r.target,
                    amount: r.minimum,
                },
            );
        } else {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| {
                    matches!(
                        a,
                        Action::Pass | Action::PutTrigger { .. } | Action::Discard { .. }
                    )
                })
                .unwrap();
            replay(g, a);
        }
    }
    panic!("no damage");
}

#[test]
fn all_thirteen_complete_bodies_cast_with_private_surveil_clue_hybrid_ward_and_printed_abilities() {
    let base = [
        (3, 3),
        (2, 2),
        (2, 3),
        (1, 2),
        (3, 4),
        (1, 2),
        (2, 2),
        (2, 1),
        (1, 2),
        (1, 4),
        (2, 2),
        (5, 3),
        (3, 3),
    ];
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Hand);
            let before = g.state.players[p.index()].library.len();
            full_cast(&mut g, id);
            assert_eq!(pt(&g, id), base[i], "{name}");
            assert_eq!(
                g.keywords(id).contains(Keywords::FLYING),
                [3, 8, 11].contains(&i),
                "{name}"
            );
            assert_eq!(g.keywords(id).contains(Keywords::FLASH), i == 11);
            assert_eq!(g.keywords(id).contains(Keywords::VIGILANCE), i == 9);
            assert!(!g.keywords(id).contains(Keywords::HEXPROOF));
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 1);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            assert_eq!(
                before - g.state.players[p.index()].library.len(),
                if i == 1 { 3 } else { 0 }
            );
            assert_eq!(
                g.state
                    .objects
                    .iter()
                    .filter(|o| o.owner == p && o.zone == Zone::Battlefield && o.card.is_token())
                    .count(),
                usize::from(i == 8)
            );
            if i == 10 {
                assert_eq!(
                    g.state.objects[id as usize]
                        .card
                        .triggers()
                        .iter()
                        .filter(|t| matches!(t.condition, TriggerCondition::Ward))
                        .count(),
                    1
                );
            }
        }
    }
}
#[test]
fn akawalli_bonuses_stack_additively_and_source_restriction_clears_at_exact_thresholds() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[0], Zone::Battlefield);
        grave(&mut g, p.other(), 9, "Forest");
        token(&mut g, p, Zone::Graveyard);
        grave(&mut g, p, 3, "Forest");
        assert_eq!(pt(&g, id), (3, 3));
        let fourth = named(&mut g, p, "Bear Cub", Zone::Graveyard);
        assert_eq!(pt(&g, id), (5, 5));
        assert!(g.keywords(id).contains(Keywords::TRAMPLE));
        assert!(!g.keywords(id).contains(Keywords::MAX_ONE_BLOCKER));
        grave(&mut g, p, 4, "Forest");
        assert_eq!(pt(&g, id), (7, 7));
        assert!(g.keywords(id).contains(Keywords::MAX_ONE_BLOCKER));
        let saved = g.snapshot();
        g.move_object(fourth, Zone::Exile);
        assert_eq!(pt(&g, id), (5, 5));
        assert!(!g.keywords(id).contains(Keywords::MAX_ONE_BLOCKER));
        g.restore(&saved).unwrap();
        assert_eq!(pt(&g, id), (7, 7));
    }
}
#[test]
fn descend_max_one_and_threshold_unblockable_enforce_actual_block_actions() {
    for p in [Player::P0, Player::P1] {
        for i in [0, 1, 6, 9] {
            let mut g = setup(p);
            let id = named(&mut g, p, NAMES[i], Zone::Battlefield);
            let a = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let b = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            if i == 6 {
                g.draw(p);
                g.draw(p);
            } else {
                grave(&mut g, p, 8, "Forest");
            }
            attack(&mut g, &[id]);
            let action = Action::Block {
                blocker: a,
                attacker: id,
            };
            if i == 0 {
                replay(&mut g, action);
                reject(
                    &mut g,
                    Action::Block {
                        blocker: b,
                        attacker: id,
                    },
                );
            } else {
                reject(&mut g, action);
            }
        }
    }
}
#[test]
fn kitesail_flying_requires_committed_attack_and_clears_after_combat_or_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[7], Zone::Battlefield);
        let ground = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        replay(&mut g, Action::Attack { card: id });
        assert!(!g.keywords(id).contains(Keywords::FLYING));
        let saved = g.snapshot();
        replay(&mut g, Action::FinishAttackers);
        assert!(g.keywords(id).contains(Keywords::FLYING));
        for _ in 0..6 {
            if g.state.pending == Pending::Blockers {
                break;
            }
            replay(&mut g, Action::Pass);
        }
        reject(
            &mut g,
            Action::Block {
                blocker: ground,
                attacker: id,
            },
        );
        damage(&mut g);
        for _ in 0..8 {
            if g.state.step == Step::Main2 {
                break;
            }
            replay(&mut g, Action::Pass);
        }
        assert!(!g.keywords(id).contains(Keywords::FLYING));
        g.restore(&saved).unwrap();
        replay(&mut g, Action::FinishAttackers);
        g.move_object(id, Zone::Hand);
        g.move_object(id, Zone::Battlefield);
        assert!(!g.keywords(id).contains(Keywords::FLYING));
    }
}
#[test]
fn modified_flying_counts_each_counter_any_equipment_only_owned_aura_and_exact_incarnation() {
    for p in [Player::P0, Player::P1] {
        for (attachment, foreign, counter, want) in [
            (None, false, None, false),
            (None, false, Some(CounterKind::PlusOne), true),
            (None, false, Some(CounterKind::MinusOne), true),
            (None, false, Some(CounterKind::Stun), true),
            (Some("Frantic Strength"), false, None, true),
            (Some("Frantic Strength"), true, None, false),
            (Some("Goldvein Pick"), true, None, true),
        ] {
            let mut g = setup(p);
            let id = named(&mut g, p, NAMES[10], Zone::Battlefield);
            g.state.objects[id as usize].growth = Stats {
                power: 2,
                toughness: 2,
            };
            g.state.objects[id as usize].used_abilities = 1;
            if let Some(c) = counter {
                g.add_counter(id, c, 1);
            }
            let attached = attachment.map(|n| {
                let a = named(
                    &mut g,
                    if foreign { p.other() } else { p },
                    n,
                    Zone::Battlefield,
                );
                g.state.objects[a as usize].attached_to = Some(g.object_ref(id));
                a
            });
            assert_eq!(g.keywords(id).contains(Keywords::FLYING), want);
            assert_eq!(g.is_modified(id), want);
            if let Some(a) = attached {
                let saved = g.snapshot();
                g.move_object(a, Zone::Graveyard);
                assert!(!g.keywords(id).contains(Keywords::FLYING));
                g.restore(&saved).unwrap();
                g.move_object(id, Zone::Exile);
                g.move_object(id, Zone::Battlefield);
                assert!(!g.keywords(id).contains(Keywords::FLYING));
            }
        }
    }
}
#[test]
fn draw_two_conditions_are_owned_live_and_reset_and_surveil_is_not_a_draw() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let fog = named(&mut g, p, NAMES[4], Zone::Battlefield);
        let june = named(&mut g, p, NAMES[6], Zone::Battlefield);
        let hawk = named(&mut g, p, NAMES[8], Zone::Battlefield);
        g.draw(p.other());
        g.draw(p.other());
        g.draw(p);
        assert_eq!(pt(&g, hawk), (1, 2));
        assert!(!g.keywords(fog).contains(Keywords::LIFELINK));
        let inkmage = named(&mut g, p, NAMES[1], Zone::Hand);
        full_cast(&mut g, inkmage);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        g.draw(p);
        assert_eq!(pt(&g, hawk), (3, 2));
        assert!(
            g.keywords(fog)
                .contains(Keywords::LIFELINK.union(Keywords::MENACE))
        );
        assert!(g.keywords(june).contains(Keywords::CANT_BE_BLOCKED));
        let saved = g.snapshot();
        g.begin_turn();
        assert_eq!(pt(&g, hawk), (1, 2));
        assert!(!g.keywords(june).contains(Keywords::CANT_BE_BLOCKED));
        g.restore(&saved).unwrap();
        assert_eq!(pt(&g, hawk), (3, 2));
    }
}
#[test]
fn grave_predicates_distinguish_lesson_from_creature_changeling_and_count_instant_sorcery_once() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let flyer = named(&mut g, p, NAMES[3], Zone::Battlefield);
        let ghitu = named(&mut g, p, NAMES[5], Zone::Battlefield);
        named(&mut g, p, "Three Tree Mascot", Zone::Graveyard);
        named(&mut g, p.other(), "Opt", Zone::Graveyard);

        assert_eq!(pt(&g, flyer), (1, 2));
        assert_eq!(pt(&g, ghitu), (1, 2));
        let lesson = Card::catalogue()
            .find(|c| c.is_supported() && c.is_standard_legal() && c.has_subtype("Lesson"))
            .unwrap();
        let l = spawn(&mut g, p, lesson, Zone::Graveyard);
        assert_eq!(pt(&g, flyer), (2, 3));
        g.move_object(l, Zone::Exile);
        assert_eq!(pt(&g, flyer), (1, 2));
        let a = named(&mut g, p, "Opt", Zone::Graveyard);
        assert_eq!(pt(&g, ghitu), (1, 2));
        named(&mut g, p, "Divination", Zone::Graveyard);
        assert_eq!(pt(&g, ghitu), (2, 2));
        assert!(g.keywords(ghitu).contains(Keywords::HASTE));
        g.move_object(a, Zone::Hand);
        assert!(!g.keywords(ghitu).contains(Keywords::HASTE));
    }
}
#[test]
fn druid_counts_owned_noncreature_tokens_but_neither_enemy_nor_graveyard_tokens() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[2], Zone::Battlefield);
        token(&mut g, p.other(), Zone::Battlefield);
        token(&mut g, p, Zone::Graveyard);
        assert_eq!(pt(&g, id), (2, 3));
        let t = token(&mut g, p, Zone::Battlefield);
        assert_eq!(pt(&g, id), (4, 3));
        assert!(g.keywords(id).contains(Keywords::TRAMPLE));
        g.move_object(t, Zone::Graveyard);
        assert_eq!(pt(&g, id), (2, 3));
    }
}
#[test]
fn sphinx_self_cast_disables_hexproof_until_turn_reset_and_only_owner_casts_disable_it() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[11], Zone::Hand);
        full_cast(&mut g, id);
        assert!(!g.keywords(id).contains(Keywords::HEXPROOF));
        g.begin_turn();
        assert!(g.keywords(id).contains(Keywords::HEXPROOF));
        g.state.players[p.other().index()].spells_cast_this_turn = 5;
        assert!(g.keywords(id).contains(Keywords::HEXPROOF));
        let snapshot = g.snapshot();
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        let bear = named(&mut g, p, "Bear Cub", Zone::Hand);
        full_cast(&mut g, bear);
        assert!(!g.keywords(id).contains(Keywords::HEXPROOF));
        g.restore(&snapshot).unwrap();
        assert!(g.keywords(id).contains(Keywords::HEXPROOF));
    }
}
#[test]
fn paladin_life_threshold_and_each_gain_counter_trigger_and_real_double_strike_damage() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[12], Zone::Battlefield);
        g.state.players[p.index()].life = 24;
        assert!(!g.keywords(id).contains(Keywords::DOUBLE_STRIKE));
        g.gain_life(p, 1);
        settle(&mut g);
        assert_eq!(pt(&g, id), (4, 4));
        assert!(g.keywords(id).contains(Keywords::DOUBLE_STRIKE));
        g.gain_life(p, 1);
        settle(&mut g);
        assert_eq!(pt(&g, id), (5, 5));
        g.state.players[p.index()].life = 24;
        assert!(!g.keywords(id).contains(Keywords::DOUBLE_STRIKE));
        g.state.players[p.index()].life = 25;
        let before = g.state.players[p.other().index()].life;
        attack(&mut g, &[id]);
        damage(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, before - 10);
    }
}
#[test]
fn june_own_turn_another_creature_cost_creates_clue_whose_draw_enables_her_restriction() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let june = named(&mut g, p, NAMES[6], Zone::Battlefield);
        let body = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.active = p.other();
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: june,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.active = p;
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: june,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        let own = g.object_ref(june);
        let enemy = g.object_ref(enemy);
        reject(&mut g, Action::ChooseAdditionalCost { object: own });
        reject(&mut g, Action::ChooseAdditionalCost { object: enemy });
        let body_ref = g.object_ref(body);
        replay(&mut g, Action::ChooseAdditionalCost { object: body_ref });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(body).zone, Zone::Graveyard);
        let clue = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| {
                o.zone == Zone::Battlefield
                    && o.card.token_info().is_some_and(|t| t.subtypes == "Clue")
            })
            .unwrap()
            .0 as CardId;
        g.draw(p);
        assert!(!g.keywords(june).contains(Keywords::CANT_BE_BLOCKED));
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: clue,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
        assert!(g.keywords(june).contains(Keywords::CANT_BE_BLOCKED));
    }
}
#[test]
fn sphinx_live_hexproof_rejects_enemy_target_and_spider_ward_remains_payable_with_modified_flying()
{
    use super::tests::cast;
    for p in [Player::P0, Player::P1] {
        for ward in [false, true] {
            let mut g = setup(p);
            let id = named(
                &mut g,
                p,
                if ward { NAMES[10] } else { NAMES[11] },
                Zone::Battlefield,
            );
            g.state.objects[id as usize].counters.plus_one = 1;
            let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
            g.state.priority = p.other();
            if !ward {
                assert!(!g.legal_actions().iter().any(|a| matches!(a,Action::Cast{card,target:Some(Target::Card(t)),..} if *card==shock&&*t==id)));
                g.state.players[p.index()].spells_cast_this_turn = 1;
            }
            cast(&mut g, shock, Some(Target::Card(id)));
            if ward {
                g.prepare_triggers();
                for _ in 0..10 {
                    if g.observe(p.other()).resolution_payment.is_some() {
                        break;
                    }
                    let a = g
                        .legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::Pass | Action::PutTrigger { .. }))
                        .unwrap();
                    replay(&mut g, a);
                }
                let payment = g.observe(p.other()).resolution_payment.unwrap();
                assert_eq!(payment.cost.generic, 2);
                reject(&mut g, Action::PayResolutionMana { payment: [0; 6] });
                let a = g
                    .legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayResolutionMana { .. }))
                    .unwrap();
                replay(&mut g, a);
            }
            settle(&mut g);
            assert_eq!(g.object_view(id).damage, 2);
            if ward {
                assert!(g.keywords(id).contains(Keywords::FLYING));
            }
        }
    }
}
#[test]
fn paladin_counter_overflow_during_real_trigger_resolution_is_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[12], Zone::Battlefield);
        g.state.objects[id as usize].counters.plus_one = u64::MAX;
        g.gain_life(p, 1);
        g.prepare_triggers();
        while g.state.stack.is_empty() {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PutTrigger { .. }))
                .unwrap();
            replay(&mut g, a);
        }
        replay(&mut g, Action::Pass);
        let snapshot = g.snapshot();
        let revision = g.revision();
        assert!(matches!(
            g.apply(g.actor().unwrap(), Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), snapshot);
        assert_eq!(g.revision(), revision);
    }
}
