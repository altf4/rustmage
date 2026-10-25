//! Ordinary paid whole-card scenarios; bounded continuation probes are labelled.
use super::assurance::big_helpers::{cast, checked, named, next_turn, settle, setup};
use super::*;
use crate::CreatureType;
const WIDOW: &str = "Black Widow, Super Spy";
const WORLDS: &str = "Worlds Within Worlds";
fn views(g: &Game) -> [serde_json::Value; 2] {
    [Player::P0, Player::P1].map(|p| {
        let mut v = g.observe(p);
        v.revision = 0;
        serde_json::to_value(v).unwrap()
    })
}
fn reject(g: &mut Game, a: Action) {
    let before = (g.snapshot(), g.revision(), g.state_bytes(), views(g));
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(
        (g.snapshot(), g.revision(), g.state_bytes(), views(g)),
        before
    );
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let rev = g.revision();
    let ix = g.legal_actions().iter().position(|v| *v == a).unwrap();
    let before = g.snapshot();
    let oldviews = views(&g);
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, a.clone()).unwrap();
    indexed.apply_index(actor, rev, ix).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert_eq!(views(&raw), views(&indexed));
    let after = raw.snapshot();
    raw.restore(&before).unwrap();
    assert_eq!(views(&raw), oldviews);
    assert_eq!(raw.apply_index(actor, rev, ix), Err(Error::StaleRevision));
    raw.apply(actor, a).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
fn main(g: &mut Game, p: Player) {
    g.state.active = p;
    g.state.priority = p;
    g.state.step = Step::Main1;
    g.state.pending = Pending::Priority;
    g.state.passes = 0;
}
fn moved(g: &mut Game, ids: &[CardId], to: Zone) {
    let before = g.death_trigger_sources();
    let events = g.move_objects_raw(ids, to);
    g.finish_zone_events(&events, &before);
}
fn clear_hand(g: &mut Game, p: Player) {
    let ids = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.zone == Zone::Hand && o.owner == p)
        .map(|(i, _)| i as CardId)
        .collect::<Vec<_>>();
    moved(g, &ids, Zone::Outside);
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..160 {
        if stop(g) {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::PutTrigger { .. }
                )
            })
            .unwrap_or_else(|| panic!("stalled {:?}", g.state.pending));
        checked(g, a);
    }
    panic!("hand state stalled {:?}", g.state.pending)
}

fn mana(g: &mut Game, p: Player, c: Card) {
    let (mut m, n) = c.cost();
    m[5] += n;
    g.state.players[p.index()].mana = m;
}
fn widow(g: &mut Game, p: Player) -> CardId {
    main(g, p);
    let c = Card::from_name(WIDOW).unwrap();
    assert!(c.is_supported());
    let s = named(g, p, WIDOW, Zone::Hand);
    mana(g, p, c);
    cast(g, s, None, &[]);
    settle(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    s
}
fn worlds(g: &mut Game, p: Player) -> CardId {
    main(g, p);
    let c = Card::from_name(WORLDS).unwrap();
    assert!(c.is_supported());
    let s = named(g, p, WORLDS, Zone::Hand);
    mana(g, p, c);
    cast(g, s, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    s
}
fn optional(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::EffectOptional { .. })
    });
}
fn hand(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::HandCardChoice { .. })
    });
}
fn pick(g: &mut Game, id: Option<CardId>) {
    let object = id.map(|id| g.object_ref(id));
    checked(g, Action::ChooseHandCard { object });
}
fn empty_library(g: &mut Game, p: Player) {
    let ids = g.state.players[p.index()].library.to_vec();
    moved(g, &ids, Zone::Outside);
}
fn library(g: &mut Game, p: Player, names: &[&str]) -> Vec<CardId> {
    empty_library(g, p);
    names
        .iter()
        .rev()
        .map(|n| named(g, p, n, Zone::Library))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}
fn hit(g: &mut Game, s: CardId, p: Player, combat: bool) {
    let source = g.damage_source(s);
    g.apply_damage_batch(&[DamageEvent {
        source,
        target: Target::Player(p),
        amount: 2,
        combat,
        preventable: true,
    }]);
    g.prepare_triggers();
}
fn decline(g: &mut Game) {
    checked(g, Action::ChooseOptional { accept: false });
}
fn done(g: &mut Game) {
    pick(g, None);
}
fn known(g: &Game, id: CardId) -> bool {
    g.state.known_hand.contains(&g.object_ref(id))
}
fn cast_exile(g: &mut Game, p: Player, id: CardId, target: Option<Target>) {
    main(g, p);
    checked(g, Action::AnnounceExile { card: id });
    if let Some(target) = target {
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..12 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, a);
    }
    panic!("unpaid exile");
}
#[test]
fn msh_widow_worlds_widow_body_payment_and_actual_unblocked_combat() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = widow(&mut g, p);
        let v = g.object_view(s);
        assert_eq!((v.power, v.toughness), (2, 1));
        assert!(v.card.is_legendary());
        for t in ["Human", "Spy", "Hero"] {
            assert!(v.has_subtype(t));
        }
        assert!(v.keywords.contains(Keywords::MENACE));
        next_turn(&mut g);
        next_turn(&mut g);
        library(&mut g, p.other(), &["Forest", "Shock"]);
        g.begin_step(Step::BeginCombat);
        until(&mut g, |g| matches!(g.state.pending, Pending::Attackers));
        checked(&mut g, Action::Attack { card: s });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| matches!(g.state.pending, Pending::Blockers));
        checked(&mut g, Action::FinishBlockers);
        optional(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.counter_count(s, CounterKind::PlusOne), 1);
    }
}
#[test]
fn msh_widow_worlds_widow_noncombat_and_creature_damage_do_not_trigger() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = widow(&mut g, p);
        hit(&mut g, s, p.other(), false);
        assert!(g.state.pending_triggers.is_empty());
        let v = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let source = g.damage_source(s);
        g.apply_damage_batch(&[DamageEvent {
            source,
            target: Target::Card(v),
            amount: 1,
            combat: true,
            preventable: true,
        }]);
        assert!(g.state.pending_triggers.is_empty());
    }
}
#[test]
fn msh_widow_worlds_widow_exact_damaged_library_lands_until_first_nonland() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = widow(&mut g, p);
        let ids = library(&mut g, p.other(), &["Forest", "Island", "Shock", "Murder"]);
        hit(&mut g, s, p.other(), true);
        optional(&mut g);
        for &id in &ids[..3] {
            assert_eq!(g.object_view(id).zone, Zone::Exile);
        }
        assert_eq!(g.object_view(ids[3]).zone, Zone::Library);
        decline(&mut g);
        settle(&mut g);
        assert!(g.can_play_exiled(p, ids[2]));
        assert!(!g.can_play_exiled(p, ids[0]));
        assert!(!g.can_play_exiled(p.other(), ids[2]));
        let row = g
            .state
            .exile_permissions
            .iter()
            .find(|r| r.object.card == ids[2])
            .unwrap();
        assert!(row.spend_any_mana && row.cast_only);
    }
}
#[test]
fn msh_widow_worlds_widow_counter_branch_preserves_counter_replacement_order() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = widow(&mut g, p);
        named(&mut g, p, "Doubling Season", Zone::Battlefield);
        let id = library(&mut g, p.other(), &["Shock"])[0];
        hit(&mut g, s, p.other(), true);
        optional(&mut g);
        checked(&mut g, Action::ChooseOptional { accept: true });
        settle(&mut g);
        assert_eq!(g.counter_count(s, CounterKind::PlusOne), 2);
        assert!(!g.can_play_exiled(p, id));
    }
}
#[test]
fn msh_widow_worlds_widow_empty_all_lands_and_source_gone() {
    for p in [Player::P0, Player::P1] {
        for names in [vec![], vec!["Forest", "Island"]] {
            let mut g = setup(p);
            let s = widow(&mut g, p);
            let ids = library(&mut g, p.other(), &names);
            hit(&mut g, s, p.other(), true);
            optional(&mut g);
            checked(&mut g, Action::ChooseOptional { accept: true });
            settle(&mut g);
            assert_eq!(g.counter_count(s, CounterKind::PlusOne), 1);
            assert!(g.state.exile_permissions.is_empty());
            for id in ids {
                assert_eq!(g.object_view(id).zone, Zone::Exile);
            }
        }
        let mut g = setup(p);
        let s = widow(&mut g, p);
        let id = library(&mut g, p.other(), &["Shock"])[0];
        hit(&mut g, s, p.other(), true);
        moved(&mut g, &[s], Zone::Graveyard);
        settle(&mut g);
        assert!(g.can_play_exiled(p, id));
    }
}
#[test]
fn msh_widow_worlds_widow_captured_controller_and_old_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = widow(&mut g, p);
        let id = library(&mut g, p.other(), &["Shock"])[0];
        hit(&mut g, s, p.other(), true);
        moved(&mut g, &[s], Zone::Exile);
        moved(&mut g, &[s], Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.counter_count(s, CounterKind::PlusOne), 0);
        assert!(g.can_play_exiled(p, id));
        let id2 = library(&mut g, p.other(), &["Murder"])[0];
        hit(&mut g, s, p.other(), true);
        g.change_controller(g.object_ref(s), p.other());
        optional(&mut g);
        assert_eq!(g.actor(), Some(p));
        decline(&mut g);
        settle(&mut g);
        assert!(g.can_play_exiled(p, id2));
    }
}
#[test]
fn msh_widow_worlds_widow_paid_foreign_any_type_spell_and_ordinary_timing() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = widow(&mut g, p);
        let id = library(&mut g, p.other(), &["Divination"])[0];
        hit(&mut g, s, p.other(), true);
        optional(&mut g);
        decline(&mut g);
        settle(&mut g);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        g.state.step = Step::EndCombat;
        g.state.priority = p;
        reject(&mut g, Action::AnnounceExile { card: id });
        let before = g.hand_count(p);
        cast_exile(&mut g, p, id, None);
        settle(&mut g);
        assert_eq!(g.hand_count(p), before + 2);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
        assert_eq!(g.object_view(id).owner, p.other());
    }
}
#[test]
fn msh_widow_worlds_widow_permission_exact_expiry_and_source_independence() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = widow(&mut g, p);
        let id = library(&mut g, p.other(), &["Shock"])[0];
        hit(&mut g, s, p.other(), true);
        optional(&mut g);
        decline(&mut g);
        settle(&mut g);
        moved(&mut g, &[s], Zone::Graveyard);
        assert!(g.can_play_exiled(p, id));
        let old = g.object_ref(id);
        moved(&mut g, &[id], Zone::Hand);
        moved(&mut g, &[id], Zone::Exile);
        assert_ne!(old, g.object_ref(id));
        assert!(!g.can_play_exiled(p, id));
        let s = widow(&mut g, p);
        let id = library(&mut g, p.other(), &["Shock"])[0];
        hit(&mut g, s, p.other(), true);
        optional(&mut g);
        decline(&mut g);
        settle(&mut g);
        next_turn(&mut g);
        assert!(!g.can_play_exiled(p, id));
    }
}
#[test]
fn msh_widow_worlds_widow_optional_owned_privacy_and_atomicity() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = widow(&mut g, p);
        library(&mut g, p.other(), &["Shock"]);
        hit(&mut g, s, p.other(), true);
        optional(&mut g);
        reject(&mut g, Action::ChooseHandCard { object: None });
        g = orphan(g, Action::ChooseOptional { accept: false });
        settle(&mut g);
        assert!(g.state.exile_permissions.iter().all(|r| r.player == p));
    }
}
#[test]
fn msh_widow_worlds_widow_cast_only_route_does_not_invent_land_play() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let land = named(&mut g, p, "Forest", Zone::Exile);
        g.grant_exile_play_permission(p, g.object_ref(land), false);
        g.state.exile_permissions.last_mut().unwrap().cast_only = true;
        main(&mut g, p);
        assert!(!g.can_play_exiled_land(p, land));
        reject(&mut g, Action::PlayLand { card: land });
        g.grant_exile_play_permission(p, g.object_ref(land), false);
        assert!(g.can_play_exiled_land(p, land));
        checked(&mut g, Action::PlayLand { card: land });
    }
}
#[test]
fn msh_widow_worlds_worlds_all_creatures_then_private_apnap_one_batch_and_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        clear_hand(&mut g, p);
        clear_hand(&mut g, p.other());
        let old = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let old2 = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let a = named(&mut g, p, "Bear Cub", Zone::Hand);
        let b = named(&mut g, p.other(), "Hill Giant", Zone::Hand);
        let spell = worlds(&mut g, p);
        hand(&mut g);
        assert_eq!(g.actor(), Some(p));
        assert_eq!(g.object_view(old).zone, Zone::Exile);
        assert_eq!(g.object_view(old2).zone, Zone::Exile);
        pick(&mut g, Some(a));
        assert_eq!(g.object_view(a).zone, Zone::Hand);
        assert!(g.observe(p.other()).hand_choice.is_none());
        assert!(!known(&g, a));
        done(&mut g);
        assert_eq!(g.actor(), Some(p.other()));
        for viewer in [p, p.other()] {
            let v = g.observe(viewer).hand_deployment.unwrap();
            assert_eq!(v.actor, Some(p.other()));
            assert_eq!(v.selected_counts[p.index()], Some(1));
            assert_eq!(v.selected_counts[p.other().index()], None);
        }
        assert!(!known(&g, a));
        pick(&mut g, Some(b));
        assert_eq!(g.object_view(a).zone, Zone::Hand);
        done(&mut g);
        settle(&mut g);
        assert!(g.on_battlefield(a) && g.on_battlefield(b));
        assert_eq!(g.object_view(a).controller, p);
        assert_eq!(g.object_view(b).controller, p.other());
        assert_eq!(g.object_view(old).zone, Zone::Hand);
        assert_eq!(g.object_view(old2).zone, Zone::Hand);
        assert!(known(&g, old) && known(&g, old2));
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
    }
}
#[test]
fn msh_widow_worlds_worlds_zero_creatures_and_zero_hand_choices_still_exiles_spell() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let spell = worlds(&mut g, p);
        hand(&mut g);
        done(&mut g);
        done(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
        assert!(g.state.resolving.is_none());
    }
}
#[test]
fn msh_widow_worlds_worlds_multiple_cards_restrictions_and_stale_references() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        clear_hand(&mut g, p);
        let a = named(&mut g, p, "Bear Cub", Zone::Hand);
        let b = named(&mut g, p, "Hill Giant", Zone::Hand);
        let land = named(&mut g, p, "Forest", Zone::Hand);
        let wrong = named(&mut g, p.other(), "Bear Cub", Zone::Hand);
        worlds(&mut g, p);
        hand(&mut g);
        for id in [land, wrong] {
            let object = g.object_ref(id);
            reject(
                &mut g,
                Action::ChooseHandCard {
                    object: Some(object),
                },
            );
        }
        pick(&mut g, Some(a));
        let object = g.object_ref(a);
        reject(
            &mut g,
            Action::ChooseHandCard {
                object: Some(object),
            },
        );
        let old = g.object_ref(b);
        moved(&mut g, &[b], Zone::Outside);
        moved(&mut g, &[b], Zone::Hand);
        reject(&mut g, Action::ChooseHandCard { object: Some(old) });
        pick(&mut g, Some(b));
        done(&mut g);
        done(&mut g);
        settle(&mut g);
        assert!(g.on_battlefield(a) && g.on_battlefield(b));
        assert_eq!(g.object_view(land).zone, Zone::Hand);
    }
}
#[test]
fn msh_widow_worlds_worlds_choices_owned_orphan_and_hidden_counterfactual() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        clear_hand(&mut g, p);
        let a = named(&mut g, p, "Bear Cub", Zone::Hand);
        let hidden = named(&mut g, p, "Shock", Zone::Hand);
        worlds(&mut g, p);
        hand(&mut g);
        let before = serde_json::to_value(g.observe(p.other())).unwrap();
        let mut counter = g.clone();
        counter.state.objects[hidden as usize].card = Card::from_name("Giant Growth").unwrap();
        assert_eq!(
            serde_json::to_value(counter.observe(p.other())).unwrap(),
            before
        );
        let r = g.object_ref(a);
        g = orphan(g, Action::ChooseHandCard { object: Some(r) });
        g = orphan(g, Action::ChooseHandCard { object: None });
        assert!(
            g.observe(p.other())
                .hand_choice
                .unwrap()
                .selected
                .is_empty()
        );
        g = orphan(g, Action::ChooseHandCard { object: None });
        settle(&mut g);
        assert!(g.on_battlefield(a));
    }
}
#[test]
fn msh_widow_worlds_worlds_entry_choice_pauses_before_originals_return() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        clear_hand(&mut g, p);
        let old = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let a = named(&mut g, p, "Adaptive Automaton", Zone::Hand);
        worlds(&mut g, p);
        hand(&mut g);
        pick(&mut g, Some(a));
        done(&mut g);
        done(&mut g);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EntryValue { .. })
        });
        assert_eq!(g.actor(), Some(p));
        assert_eq!(g.object_view(old).zone, Zone::Exile);
        assert_eq!(g.object_view(a).zone, Zone::Hand);
        assert!(g.observe(p.other()).hand_choice.is_none());
        for viewer in [p, p.other()] {
            let v = g.observe(viewer).hand_deployment.unwrap();
            assert_eq!(v.actor, None);
            assert_eq!(v.selected_counts[p.index()], Some(1));
            assert_eq!(v.selected_counts[p.other().index()], Some(0));
        }
        g = orphan(
            g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::CreatureType(CreatureType::from_name("Hero").unwrap()),
            },
        );
        settle(&mut g);
        assert!(g.has_subtype(a, "Hero"));
        assert_eq!(g.object_view(old).zone, Zone::Hand);
    }
}
#[test]
fn msh_widow_worlds_worlds_simultaneous_entry_lord_and_own_etb_observers() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        clear_hand(&mut g, p);
        named(&mut g, p, "Garruk's Uprising", Zone::Battlefield);
        let first = named(&mut g, p, "Dwynen, Gilt-Leaf Daen", Zone::Hand);
        let lord = named(&mut g, p, "Elvish Archdruid", Zone::Hand);
        worlds(&mut g, p);
        hand(&mut g);
        let before = g.hand_count(p);
        // Select the printed 3-power creature first. A sequential first entry
        // would miss Uprising; simultaneous entry sees the later Elf lord.
        pick(&mut g, Some(first));
        pick(&mut g, Some(lord));
        done(&mut g);
        done(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(first).power, 4);
        assert_eq!(g.object_view(lord).power, 3);
        assert_eq!(g.hand_count(p), before - 2 + 1);
        assert!(g.state.objects[first as usize].summoning_sick);
    }
}
#[test]
fn msh_widow_worlds_worlds_returns_owner_not_old_controller_and_no_old_counters() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let old = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[old as usize].controller = p;
        g.effect_counter(
            old,
            CounterKind::PlusOne,
            2,
            replacements::ResumeMode::Program,
        );
        let r = g.object_ref(old);
        worlds(&mut g, p);
        hand(&mut g);
        done(&mut g);
        done(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(old).zone, Zone::Hand);
        assert_eq!(g.object_view(old).owner, p.other());
        assert_ne!(r, g.object_ref(old));
        assert_eq!(g.counter_count(old, CounterKind::PlusOne), 0);
    }
}
#[test]
fn msh_widow_worlds_worlds_entry_tapped_replacement_and_no_cast_event() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        clear_hand(&mut g, p);
        named(
            &mut g,
            p.other(),
            "Authority of the Consuls",
            Zone::Battlefield,
        );
        let a = named(&mut g, p, "Bear Cub", Zone::Hand);
        worlds(&mut g, p);
        hand(&mut g);
        let count = g.state.players[p.index()].spells_cast_this_turn;
        pick(&mut g, Some(a));
        done(&mut g);
        done(&mut g);
        settle(&mut g);
        assert!(g.object_view(a).tapped);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, count);
    }
}
#[test]
fn msh_widow_worlds_worlds_effective_creatures_excludes_moon_land() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let v = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let moon = named(&mut g, p, "Imprisoned in the Moon", Zone::Hand);
        mana(
            &mut g,
            p,
            Card::from_name("Imprisoned in the Moon").unwrap(),
        );
        cast(&mut g, moon, None, &[Target::Card(v)]);
        settle(&mut g);
        assert!(!g.is_creature(v));
        let spell = worlds(&mut g, p);
        hand(&mut g);
        assert!(g.on_battlefield(v));
        done(&mut g);
        done(&mut g);
        settle(&mut g);
        assert!(g.on_battlefield(v));
        assert_eq!(g.object_view(spell).zone, Zone::Exile);
    }
}
#[test]
fn msh_widow_worlds_worlds_tokens_cease_equipment_stays_and_aura_dies() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let v = named(&mut g, p, "Hill Giant", Zone::Battlefield);
        let aura = named(&mut g, p, "Pacifism", Zone::Hand);
        mana(&mut g, p, Card::from_name("Pacifism").unwrap());
        cast(&mut g, aura, None, &[Target::Card(v)]);
        settle(&mut g);
        let equipment = named(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        g.attach_equipment(g.object_ref(equipment), g.object_ref(v));
        g.create_tokens(
            p,
            Card::from_name("@token/msh-robot-villain").unwrap(),
            1,
            false,
        );
        let token = g
            .state
            .objects
            .iter()
            .enumerate()
            .find(|(_, o)| o.zone == Zone::Battlefield && o.is_token())
            .unwrap()
            .0 as CardId;
        worlds(&mut g, p);
        hand(&mut g);
        done(&mut g);
        done(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(v).zone, Zone::Hand);
        assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
        assert!(g.on_battlefield(equipment));
        assert!(g.state.objects[equipment as usize].attached_to.is_none());
        assert_eq!(g.object_view(token).zone, Zone::Outside);
    }
}
#[test]
fn msh_widow_worlds_worlds_exact_initial_exile_does_not_recover_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let v = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        worlds(&mut g, p);
        hand(&mut g);
        // Bounded executor probe: no response priority exists during this choice.
        moved(&mut g, &[v], Zone::Graveyard);
        moved(&mut g, &[v], Zone::Exile);
        done(&mut g);
        done(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(v).zone, Zone::Exile);
    }
}
#[test]
fn msh_widow_worlds_worlds_active_player_selects_before_nonactive_spell_controller() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        worlds(&mut g, p);
        g.state.active = p.other();
        hand(&mut g);
        assert_eq!(g.actor(), Some(p.other()));
        done(&mut g);
        assert_eq!(g.actor(), Some(p));
        done(&mut g);
        settle(&mut g);
    }
}
#[test]
fn msh_widow_worlds_widow_menace_forbids_exactly_one_blocker() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = widow(&mut g, p);
        let a = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let b = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        next_turn(&mut g);
        next_turn(&mut g);
        g.begin_step(Step::BeginCombat);
        until(&mut g, |g| matches!(g.state.pending, Pending::Attackers));
        checked(&mut g, Action::Attack { card: s });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| matches!(g.state.pending, Pending::Blockers));
        checked(
            &mut g,
            Action::Block {
                blocker: a,
                attacker: s,
            },
        );
        reject(&mut g, Action::FinishBlockers);
        checked(
            &mut g,
            Action::Block {
                blocker: b,
                attacker: s,
            },
        );
        checked(&mut g, Action::FinishBlockers);
    }
}
#[test]
fn msh_widow_worlds_widow_copied_damage_packets_keep_distinct_exile_results() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = widow(&mut g, p);
        let ids = library(&mut g, p.other(), &["Shock", "Murder"]);
        hit(&mut g, s, p.other(), true);
        // Copy a real queued triggered packet, retaining its damage event.
        let mut copy = g
            .state
            .pending_triggers
            .iter()
            .chain(&g.state.ordering_triggers)
            .chain(&g.state.stack)
            .find(|i| i.card == s)
            .unwrap()
            .clone();
        copy.id = g.next_stack_id();
        copy.is_copy = true;
        g.state.stack.push(copy);
        for _ in 0..2 {
            optional(&mut g);
            decline(&mut g);
        }
        settle(&mut g);
        for id in ids {
            assert!(g.can_play_exiled(p, id));
        }
        assert_eq!(g.counter_count(s, CounterKind::PlusOne), 0);
    }
}
#[test]
fn msh_widow_worlds_widow_permission_keeps_mandatory_discard_and_failed_payment_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let s = widow(&mut g, p);
        clear_hand(&mut g, p);
        let id = library(&mut g, p.other(), &["Thrill of Possibility"])[0];
        hit(&mut g, s, p.other(), true);
        optional(&mut g);
        decline(&mut g);
        settle(&mut g);
        main(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        reject(&mut g, Action::AnnounceExile { card: id });
        assert!(g.can_play_exiled(p, id));
        let payment = named(&mut g, p, "Forest", Zone::Hand);
        checked(&mut g, Action::AnnounceExile { card: id });
        let object = g.object_ref(payment);
        checked(&mut g, Action::ChooseAdditionalCost { object });
        for _ in 0..10 {
            if g.state.announcing.is_none() {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, a);
        }
        settle(&mut g);
        assert_eq!(g.object_view(payment).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
}
#[test]
fn msh_widow_worlds_worlds_actual_paid_copy_has_independent_transaction_and_exile_cleanup() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let old = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let original = worlds(&mut g, p);
        let copy = named(&mut g, p, "Choreographed Sparks", Zone::Hand);
        mana(&mut g, p, Card::from_name("Choreographed Sparks").unwrap());
        cast(&mut g, copy, Some(0), &[Target::Spell(original)]);
        // Zero-target copies commit immediately without a retarget menu.
        hand(&mut g);
        done(&mut g);
        done(&mut g);
        until(&mut g, |g| {
            g.object_view(old).zone == Zone::Hand && g.state.resolving.is_none()
        });
        assert_eq!(g.object_view(original).zone, Zone::Stack);
        hand(&mut g);
        done(&mut g);
        done(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(original).zone, Zone::Exile);
        assert_eq!(g.object_view(old).zone, Zone::Hand);
    }
}
#[test]
fn msh_widow_worlds_worlds_source_departure_return_happens_before_hand_selection() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let old = named(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let cloak = named(&mut g, p, "Cloak and Dagger, Entwined", Zone::Hand);
        mana(
            &mut g,
            p,
            Card::from_name("Cloak and Dagger, Entwined").unwrap(),
        );
        cast(&mut g, cloak, None, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::Targets { .. })
        });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(old),
            },
        );
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::HandCardChoice { .. })
        });
        let object = g.object_ref(old);
        checked(
            &mut g,
            Action::ChooseHandCard {
                object: Some(object),
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(old).zone, Zone::Exile);
        worlds(&mut g, p);
        hand(&mut g);
        assert!(g.on_battlefield(old));
        assert_eq!(g.object_view(cloak).zone, Zone::Exile);
        done(&mut g);
        done(&mut g);
        settle(&mut g);
        assert!(g.on_battlefield(old));
        assert_eq!(g.object_view(cloak).zone, Zone::Hand);
    }
}
#[test]
fn msh_widow_worlds_actual_colored_costs_reject_generic_only_atomically() {
    for p in [Player::P0, Player::P1] {
        for name in [WIDOW, WORLDS] {
            let mut g = setup(p);
            main(&mut g, p);
            let s = named(&mut g, p, name, Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 7];
            reject(&mut g, Action::AnnounceCast { card: s });
            mana(&mut g, p, Card::from_name(name).unwrap());
            cast(&mut g, s, None, &[]);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        }
    }
}
