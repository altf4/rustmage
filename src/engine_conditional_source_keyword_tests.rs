use super::tests::{act, scenario, spawn};
use super::*;
const NAMES: [&str; 9] = [
    "Barrow Naughty",
    "Didact Echo",
    "Dreadwing Scavenger",
    "Echo of Dusk",
    "Goblin Tomb Raider",
    "Gravblade Heavy",
    "Kargan Dragonrider",
    "Spineseeker Centipede",
    "Wildfire Wickerfolk",
];
const BASE: [(i128, i128); 9] = [
    (1, 3),
    (3, 2),
    (2, 2),
    (2, 2),
    (1, 2),
    (3, 4),
    (2, 2),
    (2, 1),
    (3, 2),
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
        choices.sort_by_key(|a| !matches!(a, Action::ChooseSearch { .. }));
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
    pay(g);
    settle(g);
}
fn grave(g: &mut Game, p: Player, n: usize, name: &str) -> Vec<CardId> {
    (0..n).map(|_| named(g, p, name, Zone::Graveyard)).collect()
}
fn delirium(g: &mut Game, p: Player) -> Vec<CardId> {
    ["Forest", "Opt", "Scorpion Sentinel"]
        .map(|n| named(g, p, n, Zone::Graveyard))
        .to_vec()
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
fn all_nine_whole_bodies_cast_and_keep_intrinsic_keywords_draw_loot_and_search() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Hand);
            let before = g.state.players[p.index()].library.len();
            full_cast(&mut g, id);
            assert_eq!(pt(&g, id), BASE[i], "{name}");
            let k = g.keywords(id);
            assert_eq!(k.contains(Keywords::FLYING), [0, 2].contains(&i));
            assert_eq!(k.contains(Keywords::HASTE), i == 8);
            for flag in [
                Keywords::LIFELINK,
                Keywords::DEATHTOUCH,
                Keywords::TRAMPLE,
                Keywords::VIGILANCE,
            ] {
                assert!(!k.contains(flag));
            }
            assert_eq!(
                g.state.players[p.index()].library.len(),
                before - usize::from([1, 2, 7].contains(&i)),
                "{name}"
            );
            assert_eq!(g.hand_count(p), usize::from([1, 7].contains(&i)));
            if i == 7 {
                let o = g.observe(p.other());
                assert_eq!(o.known_hand.len(), 1);
                assert_eq!(
                    o.objects
                        .iter()
                        .filter(|o| o.owner == p && o.zone == Zone::Hand)
                        .count(),
                    1
                );
            }
            assert_eq!(g.observe(p).schema, 51);
        }
    }
}
#[test]
fn barrow_naughty_excludes_itself_and_counts_owned_changeling_as_another_faerie() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[0], Zone::Battlefield);
        named(&mut g, p.other(), "Faerie Dreamthief", Zone::Battlefield);
        named(&mut g, p, "Faerie Dreamthief", Zone::Graveyard);
        assert!(!g.keywords(id).contains(Keywords::LIFELINK));
        let witness = named(&mut g, p, "Three Tree Mascot", Zone::Battlefield);
        assert!(g.keywords(id).contains(Keywords::LIFELINK));
        let s = g.snapshot();
        g.move_object(witness, Zone::Hand);
        assert!(!g.keywords(id).contains(Keywords::LIFELINK));
        g.restore(&s).unwrap();
        assert!(g.keywords(id).contains(Keywords::LIFELINK));
        assert!(g.keywords(id).contains(Keywords::FLYING));
    }
}
#[test]
fn dragonrider_uses_live_owned_dragon_subtype_and_does_not_treat_all_creatures_as_dragons() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[6], Zone::Battlefield);
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        named(&mut g, p.other(), "Three Tree Mascot", Zone::Battlefield);
        named(&mut g, p, "Three Tree Mascot", Zone::Hand);
        assert!(!g.keywords(id).contains(Keywords::FLYING));
        let dragon = named(&mut g, p, "Three Tree Mascot", Zone::Battlefield);
        assert!(g.keywords(id).contains(Keywords::FLYING));
        g.move_object(dragon, Zone::Graveyard);
        assert!(!g.keywords(id).contains(Keywords::FLYING));
    }
}
#[test]
fn artifact_witnesses_enable_both_stats_and_keywords_without_global_or_foreign_grants() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tomb = named(&mut g, p, NAMES[4], Zone::Battlefield);
        let heavy = named(&mut g, p, NAMES[5], Zone::Battlefield);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        token(&mut g, p.other(), Zone::Battlefield);
        named(&mut g, p, "Scorpion Sentinel", Zone::Graveyard);
        assert_eq!(pt(&g, tomb), (1, 2));
        assert!(!g.keywords(heavy).contains(Keywords::DEATHTOUCH));
        let t = token(&mut g, p, Zone::Battlefield);
        assert_eq!(pt(&g, tomb), (2, 2));
        assert_eq!(pt(&g, heavy), (4, 4));
        assert!(g.keywords(tomb).contains(Keywords::HASTE));
        assert!(g.keywords(heavy).contains(Keywords::DEATHTOUCH));
        assert!(!g.keywords(bear).contains(Keywords::HASTE));
        assert_eq!(pt(&g, bear), (2, 2));
        let s = g.snapshot();
        g.move_object(t, Zone::Graveyard);
        assert_eq!(pt(&g, tomb), (1, 2));
        assert!(!g.keywords(tomb).contains(Keywords::HASTE));
        g.restore(&s).unwrap();
        assert_eq!(pt(&g, heavy), (4, 4));
    }
}
#[test]
fn descend_source_grants_require_owned_permanent_cards_and_drop_immediately_when_one_leaves() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let echo = named(&mut g, p, NAMES[1], Zone::Battlefield);
        let dusk = named(&mut g, p, NAMES[3], Zone::Battlefield);
        grave(&mut g, p, 8, "Opt");
        grave(&mut g, p.other(), 4, "Forest");
        token(&mut g, p, Zone::Graveyard);
        let ids = grave(&mut g, p, 3, "Forest");
        assert!(!g.keywords(echo).contains(Keywords::FLYING));
        assert!(!g.keywords(dusk).contains(Keywords::LIFELINK));
        let fourth = named(&mut g, p, "Scorpion Sentinel", Zone::Graveyard);
        assert!(g.keywords(echo).contains(Keywords::FLYING));
        assert!(g.keywords(dusk).contains(Keywords::LIFELINK));
        assert_eq!(pt(&g, dusk), (3, 3));
        let s = g.snapshot();
        g.move_object(ids[0], Zone::Exile);
        assert!(!g.keywords(echo).contains(Keywords::FLYING));
        assert_eq!(pt(&g, dusk), (2, 2));
        g.restore(&s).unwrap();
        g.move_object(fourth, Zone::Hand);
        assert!(!g.keywords(dusk).contains(Keywords::LIFELINK));
    }
}
#[test]
fn delirium_counts_distinct_card_types_with_multitype_cards_once_and_excludes_tokens_and_opponents()
{
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let spine = named(&mut g, p, NAMES[7], Zone::Battlefield);
        let wicker = named(&mut g, p, NAMES[8], Zone::Battlefield);
        grave(&mut g, p, 7, "Forest");
        grave(&mut g, p.other(), 1, "Opt");
        token(&mut g, p, Zone::Graveyard);
        named(&mut g, p, "Scorpion Sentinel", Zone::Graveyard);
        assert_eq!(pt(&g, spine), (2, 1));
        assert!(!g.keywords(spine).contains(Keywords::VIGILANCE));
        let opt = named(&mut g, p, "Opt", Zone::Graveyard);
        assert_eq!(pt(&g, spine), (3, 3));
        assert_eq!(pt(&g, wicker), (4, 3));
        assert!(g.keywords(spine).contains(Keywords::VIGILANCE));
        assert!(g.keywords(wicker).contains(Keywords::TRAMPLE));
        g.move_object(opt, Zone::Exile);
        assert_eq!(pt(&g, wicker), (3, 2));
        assert!(!g.keywords(wicker).contains(Keywords::TRAMPLE));
        assert!(g.keywords(wicker).contains(Keywords::HASTE));
    }
}
#[test]
fn conditional_haste_changes_real_same_turn_attack_legality_and_restores_exactly() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[4], Zone::Hand);
        full_cast(&mut g, id);
        assert!(g.object_view(id).summoning_sick);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        reject(&mut g, Action::Attack { card: id });
        let witness = token(&mut g, p, Zone::Battlefield);
        let s = g.snapshot();
        assert!(g.legal_actions().contains(&Action::Attack { card: id }));
        g.move_object(witness, Zone::Graveyard);
        reject(&mut g, Action::Attack { card: id });
        g.restore(&s).unwrap();
        replay(&mut g, Action::Attack { card: id });
        replay(&mut g, Action::FinishAttackers);
        assert!(g.object_view(id).tapped);
    }
}
#[test]
fn conditional_flying_changes_real_blocking_while_reach_still_blocks() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, NAMES[6], Zone::Battlefield);
        let dragon = named(&mut g, p, "Three Tree Mascot", Zone::Battlefield);
        let ground = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let reach = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[reach as usize].gained_keywords = Keywords::REACH;
        attack(&mut g, &[id]);
        reject(
            &mut g,
            Action::Block {
                blocker: ground,
                attacker: id,
            },
        );
        assert!(g.legal_actions().contains(&Action::Block {
            blocker: reach,
            attacker: id
        }));
        let s = g.snapshot();
        g.move_object(dragon, Zone::Hand);
        replay(
            &mut g,
            Action::Block {
                blocker: ground,
                attacker: id,
            },
        );
        g.restore(&s).unwrap();
        replay(
            &mut g,
            Action::Block {
                blocker: reach,
                attacker: id,
            },
        );
    }
}
#[test]
fn real_combat_applies_conditional_lifelink_and_deathtouch_for_both_seats() {
    for p in [Player::P0, Player::P1] {
        for on in [false, true] {
            let mut g = setup(p);
            if on {
                grave(&mut g, p, 4, "Forest");
            }
            let echo = named(&mut g, p, NAMES[3], Zone::Battlefield);
            attack(&mut g, &[echo]);
            damage(&mut g);
            assert_eq!(g.state.players[p.index()].life, if on { 23 } else { 20 });
            assert_eq!(
                g.state.players[p.other().index()].life,
                if on { 17 } else { 18 }
            );
            let mut g = setup(p);
            if on {
                token(&mut g, p, Zone::Battlefield);
            }
            let heavy = named(&mut g, p, NAMES[5], Zone::Battlefield);
            let rhino = named(&mut g, p.other(), "Rough Rhino Cavalry", Zone::Battlefield);
            attack(&mut g, &[heavy]);
            replay(
                &mut g,
                Action::Block {
                    blocker: rhino,
                    attacker: heavy,
                },
            );
            damage(&mut g);
            assert_eq!(g.object_view(heavy).zone, Zone::Graveyard);
            assert_eq!(
                g.object_view(rhino).zone,
                if on {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
    }
}
#[test]
fn delirium_trample_assignment_and_vigilance_attack_use_the_same_live_predicate() {
    for p in [Player::P0, Player::P1] {
        for on in [false, true] {
            let mut g = setup(p);
            if on {
                delirium(&mut g, p);
            }
            let wicker = named(&mut g, p, NAMES[8], Zone::Battlefield);
            let spine = named(&mut g, p, NAMES[7], Zone::Battlefield);
            let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            attack(&mut g, &[wicker, spine]);
            assert_eq!(g.object_view(spine).tapped, !on);
            assert!(g.object_view(wicker).tapped);
            replay(
                &mut g,
                Action::Block {
                    blocker: bear,
                    attacker: wicker,
                },
            );
            damage(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].life,
                if on { 15 } else { 18 }
            );
            assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        }
    }
}
#[test]
fn barrow_paid_temporary_pump_preserves_conditional_lifelink_and_expires_normally() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, NAMES[0], Zone::Battlefield);
        let faerie = named(&mut g, p, "Three Tree Mascot", Zone::Battlefield);
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(&mut g, Action::PayCosts { payment: [0; 6] });
        pay(&mut g);
        settle(&mut g);
        assert_eq!(pt(&g, source), (2, 3));
        assert!(g.keywords(source).contains(Keywords::LIFELINK));
        let s = g.snapshot();
        g.move_object(faerie, Zone::Hand);
        assert!(!g.keywords(source).contains(Keywords::LIFELINK));
        assert_eq!(pt(&g, source), (2, 3));
        g.restore(&s).unwrap();
        g.finish_cleanup();
        assert_eq!(pt(&g, source), (1, 3));
        assert!(g.keywords(source).contains(Keywords::LIFELINK));
    }
}
#[test]
fn scavenger_owned_entry_and_attack_loot_complete_before_live_threshold_is_rechecked() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        grave(&mut g, p, 6, "Forest");
        let id = named(&mut g, p, NAMES[2], Zone::Hand);
        full_cast(&mut g, id);
        assert_eq!(pt(&g, id), (3, 3));
        assert!(g.keywords(id).contains(Keywords::DEATHTOUCH));
        assert_eq!(g.hand_count(p), 0);
        let before = g.state.players[p.index()].cards_drawn_this_turn;
        let hidden = g.observe(p.other());
        assert!(
            hidden
                .objects
                .iter()
                .all(|o| o.zone != Zone::Hand || o.owner != p)
        );
        g.state.objects[id as usize].summoning_sick = false; // Next-own-turn attack fixture after the actual entry test.
        attack(&mut g, &[id]);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 1);
        assert_eq!(g.hand_count(p), 0);
        assert_eq!(pt(&g, id), (3, 3));
    }
}
#[test]
fn conditional_lifelink_overflow_rolls_back_the_entire_real_damage_action() {
    let p = Player::P0;
    let mut g = setup(p);
    let source = named(&mut g, p, NAMES[0], Zone::Battlefield);
    named(&mut g, p, "Three Tree Mascot", Zone::Battlefield);
    g.state.players[0].life = i128::MAX;
    attack(&mut g, &[source]);
    replay(&mut g, Action::FinishBlockers);
    replay(&mut g, Action::Pass);
    let s = g.snapshot();
    let r = g.revision();
    let actor = g.actor().unwrap();
    for _ in 0..2 {
        assert!(matches!(
            g.apply(actor, Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), s);
        assert_eq!(g.revision(), r);
        assert_eq!(g.state.players[1].life, 20);
    }
}
