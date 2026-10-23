//! Paid combat-card scenarios with owned raw/index rollback and both observations.
use super::ability_timeline_tests::{checked, named, next_turn, settle, settle_on, setup};
use super::*;
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    checked(g, Action::AnnounceCast { card: id });
    if matches!(g.state.pending, Pending::SpellCost { .. }) {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseSpellCost { .. }))
            .unwrap();
        checked(g, a);
    }
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    let original = g.state.announcing.as_ref().map(|a| a.item.id);
    for _ in 0..8 {
        if g.state.announcing.as_ref().map(|a| a.item.id) != original
            || g.state.announcing.is_none()
        {
            break;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
    id
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(before, g.snapshot());
    assert_eq!(revision, g.revision());
}
fn activate_ability(g: &mut Game, id: CardId, ability: u16, targets: &[Target]) {
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    for &target in targets {
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(g, a);
    }
    panic!("unpaid activation");
}
fn until(g: &mut Game, condition: impl Fn(&Game) -> bool) {
    for _ in 0..100 {
        if condition(g) {
            return;
        }
        g.prepare_triggers();
        if condition(g) {
            return;
        }
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::FinishTargets
                        | Action::PayCosts { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            })
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        checked(g, a);
    }
    panic!("condition not reached {:?}", g.state.pending);
}
fn combat(g: &mut Game, id: CardId) {
    until(g, |g| g.state.pending == Pending::Attackers);
    checked(g, Action::Attack { card: id });
    checked(g, Action::FinishAttackers);
    until(g, |g| g.state.pending == Pending::Blockers);
}
fn own_main(g: &mut Game, p: Player) {
    until(g, |g| {
        g.state.active == p
            && g.state.step == Step::Main1
            && g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
    });
    for player in &mut g.state.players {
        player.mana = [30; 6];
    }
}
fn activate(g: &mut Game, id: CardId, targets: &[Target]) {
    activate_ability(g, id, 0, targets);
}

#[test]
fn tmt_combat_batch_paid_bodies_and_mutual_menace_restriction() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let b = paid(&mut g, p, "Bebop, Warthog Warrior", &[]);
        settle(&mut g);
        let r = paid(&mut g, p, "Rocksteady, Crash Courser", &[]);
        settle(&mut g);
        assert!(g.keywords(b).contains(Keywords::MENACE));
        assert!(g.keywords(r).contains(Keywords::MENACE));
        assert!(g.keywords(b).contains(Keywords::MAX_ONE_BLOCKER));
        assert!(g.keywords(r).contains(Keywords::MAX_ONE_BLOCKER));
        next_turn(&mut g);
        next_turn(&mut g);
        own_main(&mut g, p);
        let defender = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
        combat(&mut g, b);
        assert!(!g.legal_actions().contains(&Action::Block {
            attacker: b,
            blocker: defender
        }));
    }
}
#[test]
fn tmt_combat_batch_ninja_paid_noncombat_damage_retains_red_until_turn_end() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let n = paid(&mut g, p, "Raphael, Ninja Destroyer", &[]);
        settle(&mut g);
        g.state.players[p.index()].mana = [0; 6];
        g.state.players[p.other().index()].mana = [30; 6];
        g.state.priority = p.other();
        paid(&mut g, p.other(), "Shock", &[Target::Card(n)]);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].mana[3], 2);
        until(&mut g, |g| g.state.step == Step::End);
        assert_eq!(g.state.players[p.index()].mana[3], 2);
        next_turn(&mut g);
        assert_eq!(g.state.players[p.index()].mana[3], 0);
    }
}
#[test]
fn tmt_combat_batch_siblings_first_phase_only_and_untaps_attacker() {
    let p = Player::P0;
    let mut g = setup(p);
    let r = paid(&mut g, p, "Raph & Leo, Sibling Rivals", &[]);
    settle(&mut g);
    next_turn(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p);
    until(&mut g, |g| g.state.pending == Pending::Attackers);
    checked(&mut g, Action::Attack { card: r });
    checked(&mut g, Action::FinishAttackers);
    settle_on(&mut g, Some(Target::Card(r)));
    assert!(!g.state.objects[r as usize].tapped);
    assert_eq!(g.state.combat_rules.as_ref().unwrap().additional_combats, 1);
    until(&mut g, |g| g.state.pending == Pending::Attackers);
    assert_eq!(g.state.combat_rules.as_ref().unwrap().phases_started, 2);
    checked(&mut g, Action::Attack { card: r });
    checked(&mut g, Action::FinishAttackers);
    assert!(g.state.pending_triggers.is_empty());
    assert_eq!(g.state.combat_rules.as_ref().unwrap().additional_combats, 0);
}
#[test]
fn tmt_combat_batch_attitude_alliance_exile_and_paid_one_play() {
    let p = Player::P0;
    let mut g = setup(p);
    let r = paid(&mut g, p, "Raphael, Most Attitude", &[]);
    settle(&mut g);
    let shock = named(&mut g, p, "Shock", Zone::Library);
    paid(&mut g, p, "Grizzly Bears", &[]);
    settle(&mut g);
    assert_eq!(g.state.objects[shock as usize].zone, Zone::Exile);
    assert!(!g.can_cast_exiled(p, shock));
    next_turn(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p);
    combat(&mut g, r);
    checked(&mut g, Action::FinishBlockers);
    g.state.players[p.index()].mana = [30; 6];
    assert!(g.can_cast_exiled(p, shock));
    checked(&mut g, Action::AnnounceExile { card: shock });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, a);
    }
    assert_eq!(
        g.state.limited_exile_plays.as_ref().unwrap().grants[0].remaining,
        0
    );
    settle(&mut g);
}
#[test]
fn tmt_combat_batch_typecycling_actual_discard_search_privacy_and_fail_find() {
    for (name, land, wrong) in [
        ("Bebop, Warthog Warrior", "Swamp", "Forest"),
        ("Rocksteady, Crash Courser", "Forest", "Swamp"),
    ] {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let card = Card::from_name(name).unwrap();
            let cycler = super::tests::spawn(&mut g, p, card, Zone::Hand);
            let found = named(&mut g, p, land, Zone::Library);
            let wrong = named(&mut g, p, wrong, Zone::Library);
            let before = g.state.players[p.index()].mana.iter().sum::<u32>();
            activate(&mut g, cycler, &[]);
            assert_eq!(g.state.objects[cycler as usize].zone, Zone::Graveyard);
            assert_eq!(
                g.state.players[p.index()].mana.iter().sum::<u32>(),
                before - 2
            );
            until(&mut g, |g| {
                matches!(g.state.pending, Pending::EffectSearch { .. })
            });
            assert!(g.observe(p.other()).actions.is_empty());
            reject(&mut g, Action::ChooseSearch { card: wrong });
            checked(&mut g, Action::ChooseSearch { card: found });
            settle(&mut g);
            assert_eq!(g.state.objects[found as usize].zone, Zone::Hand);
            assert!(
                g.observe(p.other())
                    .known_hand
                    .contains(&g.object_ref(found))
            );
        }
    }
}
#[test]
fn tmt_combat_batch_ninja_lethal_actual_spell_still_produces_mana() {
    let p = Player::P0;
    let mut g = setup(p);
    let n = paid(&mut g, p, "Raphael, Ninja Destroyer", &[]);
    settle(&mut g);
    g.state.players[p.index()].mana = [0; 6];
    g.state.players[p.index()].mana = [30; 6];
    paid(&mut g, p, "Bot Bashing Time", &[Target::Card(n)]);
    g.state.players[p.index()].mana = [0; 6];
    settle(&mut g);
    assert_eq!(g.state.objects[n as usize].zone, Zone::Exile);
    assert_eq!(g.state.players[p.index()].mana[3], 6);
}
#[test]
fn tmt_combat_batch_ninja_required_block_and_simultaneous_two_dealers_sum_once() {
    let p = Player::P0;
    let mut g = setup(p);
    let n = paid(&mut g, p, "Raphael, Ninja Destroyer", &[]);
    settle(&mut g);
    let a = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let b = named(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    next_turn(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p);
    combat(&mut g, n);
    reject(&mut g, Action::FinishBlockers);
    checked(
        &mut g,
        Action::Block {
            blocker: a,
            attacker: n,
        },
    );
    checked(
        &mut g,
        Action::Block {
            blocker: b,
            attacker: n,
        },
    );
    g.state.players[p.index()].mana = [0; 6];
    checked(&mut g, Action::FinishBlockers);
    until(&mut g, |g| g.state.players[p.index()].mana[3] == 4);
    assert_eq!(g.state.objects[n as usize].zone, Zone::Graveyard);
    assert_eq!(g.state.players[p.index()].mana[3], 4);
}
#[test]
fn tmt_combat_batch_siblings_all_targets_illegal_prevents_extra_phase() {
    let p = Player::P0;
    let mut g = setup(p);
    let r = paid(&mut g, p, "Raph & Leo, Sibling Rivals", &[]);
    settle(&mut g);
    next_turn(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p);
    until(&mut g, |g| g.state.pending == Pending::Attackers);
    checked(&mut g, Action::Attack { card: r });
    checked(&mut g, Action::FinishAttackers);
    until(&mut g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(r),
        })
    });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(r),
        },
    );
    checked(&mut g, Action::FinishTargets);
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, a);
    }
    g.state.players[p.index()].mana = [30; 6];
    paid(&mut g, p, "Unsummon", &[Target::Card(r)]);
    settle(&mut g);
    assert_eq!(g.state.objects[r as usize].zone, Zone::Hand);
    assert_eq!(g.state.combat_rules.as_ref().unwrap().additional_combats, 0);
}
#[test]
fn tmt_combat_batch_attitude_optional_decline_and_expiry() {
    let p = Player::P0;
    let mut g = setup(p);
    let r = paid(&mut g, p, "Raphael, Most Attitude", &[]);
    settle(&mut g);
    let top = named(&mut g, p, "Shock", Zone::Library);
    paid(&mut g, p, "Grizzly Bears", &[]);
    until(&mut g, |g| {
        g.legal_actions()
            .contains(&Action::ChooseOptional { accept: false })
    });
    checked(&mut g, Action::ChooseOptional { accept: false });
    settle(&mut g);
    assert_eq!(g.state.objects[top as usize].zone, Zone::Library);
    paid(&mut g, p, "Grizzly Bears", &[]);
    settle(&mut g);
    assert_eq!(g.state.objects[top as usize].zone, Zone::Exile);
    next_turn(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p);
    combat(&mut g, r);
    assert!(g.can_cast_exiled(p, top));
    next_turn(&mut g);
    assert!(!g.can_cast_exiled(p, top));
    assert_eq!(g.state.objects[top as usize].zone, Zone::Exile);
}
#[test]
fn tmt_combat_batch_attitude_allows_later_exile_and_paid_land_once() {
    let p = Player::P0;
    let mut g = setup(p);
    let r = paid(&mut g, p, "Raphael, Most Attitude", &[]);
    settle(&mut g);
    next_turn(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p);
    combat(&mut g, r);
    until(&mut g, |g| {
        g.state.step == Step::Main2 && g.state.pending == Pending::Priority
    });
    g.state.players[p.index()].mana = [30; 6];
    let land = named(&mut g, p, "Forest", Zone::Library);
    paid(&mut g, p, "Grizzly Bears", &[]);
    settle(&mut g);
    assert_eq!(g.state.objects[land as usize].zone, Zone::Exile);
    checked(&mut g, Action::PlayLand { card: land });
    assert_eq!(g.state.objects[land as usize].zone, Zone::Battlefield);
    assert_eq!(
        g.state.limited_exile_plays.as_ref().unwrap().grants[0].remaining,
        0
    );
}
#[test]
fn tmt_combat_batch_attitude_permission_survives_source_leaving() {
    let p = Player::P0;
    let mut g = setup(p);
    let r = paid(&mut g, p, "Raphael, Most Attitude", &[]);
    settle(&mut g);
    let top = named(&mut g, p, "Shock", Zone::Library);
    paid(&mut g, p, "Grizzly Bears", &[]);
    settle(&mut g);
    next_turn(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p);
    combat(&mut g, r);
    checked(&mut g, Action::FinishBlockers);
    g.state.players[p.index()].mana = [30; 6];
    paid(&mut g, p, "Unsummon", &[Target::Card(r)]);
    settle(&mut g);
    assert_eq!(g.state.objects[r as usize].zone, Zone::Hand);
    assert!(g.can_cast_exiled(p, top));
    checked(&mut g, Action::AnnounceExile { card: top });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    until(&mut g, |g| g.state.announcing.is_none());
    assert_eq!(
        g.state.limited_exile_plays.as_ref().unwrap().grants[0].remaining,
        0
    );
}
#[test]
fn tmt_combat_batch_siblings_two_attacking_targets_partial_fizzle() {
    let p = Player::P0;
    let mut g = setup(p);
    let r = paid(&mut g, p, "Raph & Leo, Sibling Rivals", &[]);
    settle(&mut g);
    let bear = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let spectator = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    next_turn(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p);
    until(&mut g, |g| g.state.pending == Pending::Attackers);
    checked(&mut g, Action::Attack { card: r });
    checked(&mut g, Action::Attack { card: bear });
    checked(&mut g, Action::FinishAttackers);
    until(&mut g, |g| {
        g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(r),
        })
    });
    reject(&mut g, Action::FinishTargets);
    reject(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(spectator),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(r),
        },
    );
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    until(&mut g, |g| g.state.announcing.is_none());
    g.state.players[p.index()].mana = [30; 6];
    paid(&mut g, p, "Unsummon", &[Target::Card(bear)]);
    settle(&mut g);
    assert!(!g.state.objects[r as usize].tapped);
    assert_eq!(g.state.objects[bear as usize].zone, Zone::Hand);
    assert_eq!(g.state.combat_rules.as_ref().unwrap().additional_combats, 1);
}
#[test]
fn tmt_combat_batch_typecycling_can_fail_to_find() {
    for name in ["Bebop, Warthog Warrior", "Rocksteady, Crash Courser"] {
        let p = Player::P0;
        let mut g = setup(p);
        let c = super::tests::spawn(&mut g, p, Card::from_name(name).unwrap(), Zone::Hand);
        let library = g.state.players[p.index()].library.len();
        activate(&mut g, c, &[]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        checked(&mut g, Action::FinishSearch);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), library);
        assert_eq!(g.state.objects[c as usize].zone, Zone::Graveyard);
    }
}
#[test]
fn tmt_combat_batch_ninja_retained_mana_overflow_is_transactional() {
    let p = Player::P0;
    let mut g = setup(p);
    let n = paid(&mut g, p, "Raphael, Ninja Destroyer", &[]);
    settle(&mut g);
    paid(&mut g, p, "Shock", &[Target::Card(n)]);
    until(&mut g, |g| {
        g.state
            .stack
            .last()
            .is_some_and(|s| s.damage_event.is_some())
    });
    g.state.players[p.index()].mana = [0, 0, 0, u32::MAX, 0, 0];
    checked(&mut g, Action::Pass);
    reject(&mut g, Action::Pass);
}
#[test]
fn tmt_combat_batch_attitude_countered_paid_spell_does_not_refund_play() {
    let p = Player::P0;
    let mut g = setup(p);
    let r = paid(&mut g, p, "Raphael, Most Attitude", &[]);
    settle(&mut g);
    let top = named(&mut g, p, "Shock", Zone::Library);
    paid(&mut g, p, "Grizzly Bears", &[]);
    settle(&mut g);
    next_turn(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p);
    combat(&mut g, r);
    checked(&mut g, Action::FinishBlockers);
    for player in &mut g.state.players {
        player.mana = [30; 6];
    }
    checked(&mut g, Action::AnnounceExile { card: top });
    checked(
        &mut g,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    until(&mut g, |g| g.state.announcing.is_none());
    if g.actor() == Some(p) {
        checked(&mut g, Action::Pass);
    }
    paid(&mut g, p.other(), "Negate", &[Target::Spell(top)]);
    settle(&mut g);
    assert_eq!(g.state.objects[top as usize].zone, Zone::Graveyard);
    assert_eq!(
        g.state.limited_exile_plays.as_ref().unwrap().grants[0].remaining,
        0
    );
}
#[test]
fn tmt_combat_batch_first_combat_without_attacks_still_counts() {
    let p = Player::P0;
    let mut g = setup(p);
    let r = paid(&mut g, p, "Raph & Leo, Sibling Rivals", &[]);
    settle(&mut g);
    next_turn(&mut g);
    next_turn(&mut g);
    own_main(&mut g, p);
    g.add_combat();
    until(&mut g, |g| g.state.pending == Pending::Attackers);
    checked(&mut g, Action::FinishAttackers);
    until(&mut g, |g| g.state.pending == Pending::Attackers);
    assert_eq!(g.state.combat_rules.as_ref().unwrap().phases_started, 2);
    checked(&mut g, Action::Attack { card: r });
    checked(&mut g, Action::FinishAttackers);
    assert!(g.state.pending_triggers.is_empty());
    assert!(g.state.announcing.is_none());
}
#[test]
fn tmt_combat_batch_bebop_grant_filters_controller_and_source_departure() {
    let p = Player::P0;
    let mut g = setup(p);
    let b = paid(&mut g, p, "Bebop, Warthog Warrior", &[]);
    settle(&mut g);
    let r = paid(&mut g, p, "Rocksteady, Crash Courser", &[]);
    settle(&mut g);
    let enemy = super::tests::spawn(
        &mut g,
        p.other(),
        Card::from_name("Rocksteady, Crash Courser").unwrap(),
        Zone::Battlefield,
    );
    assert!(!g.keywords(enemy).contains(Keywords::MENACE));
    assert_eq!((g.stats(b).power, g.stats(b).toughness), (5, 4));
    assert_eq!((g.stats(r).power, g.stats(r).toughness), (7, 7));
    paid(&mut g, p, "Unsummon", &[Target::Card(b)]);
    settle(&mut g);
    assert!(!g.keywords(r).contains(Keywords::MENACE));
    assert!(g.keywords(r).contains(Keywords::MAX_ONE_BLOCKER));
}
