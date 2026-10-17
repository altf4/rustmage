use super::tests::{scenario, spawn};
use super::*;

fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    let card = Card::from_name(name).unwrap();
    assert!(card.is_supported(), "{name}");
    spawn(g, p, card, zone)
}
fn checked(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let before = g.snapshot();
    assert_eq!(
        g.apply(actor.other(), action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(g.snapshot(), before);
    let mut branch = {
        let parent = g.clone();
        parent.clone()
    };
    branch.apply_index(actor, revision, index).unwrap();
    g.apply(actor, action.clone()).unwrap();
    assert_eq!(g.snapshot(), branch.snapshot());
    for player in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(player)).unwrap(),
            serde_json::to_value(branch.observe(player)).unwrap()
        );
    }
    let after = g.snapshot();
    g.restore(&before).unwrap();
    assert_eq!(
        g.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    g.apply(actor, action).unwrap();
    assert_eq!(g.snapshot(), after);
}
fn settle_on(g: &mut Game, chosen: Option<Target>) {
    g.prepare_triggers();
    for _ in 0..100 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let action = if let Some(target) =
            chosen.filter(|t| actions.contains(&Action::ChooseTarget { target: *t }))
        {
            Action::ChooseTarget { target }
        } else if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .or_else(|| actions.iter().find(|a| matches!(a, Action::FinishTargets)))
                .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                .unwrap()
                .clone()
        };
        checked(g, action);
    }
    panic!("unsettled {:?}", g.state.pending);
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    g.state.players[p.index()].mana = [30; 6];
    g
}

fn settle(g: &mut Game) {
    settle_on(g, None);
}
fn reject(g: &mut Game, a: Action) {
    let before = g.snapshot();
    let rev = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), rev);
}
fn cast(g: &mut Game, id: CardId, mode: Option<u16>, targets: &[Target]) {
    checked(g, Action::AnnounceCast { card: id });
    if let Some(mode) = mode {
        reject(g, Action::ChooseMode { mode: 3 });
        checked(g, Action::ChooseMode { mode });
    }
    for &target in targets {
        for _ in 0..3 {
            if g.legal_actions().contains(&Action::ChooseTarget { target }) {
                break;
            }
            checked(g, Action::FinishTargets);
        }
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
            })
            .unwrap();
        checked(g, a);
    }
    panic!("unpaid");
}
fn activate(g: &mut Game, card: CardId) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::AnnounceActivate { card: id, ability: 0, .. } if *id==card))
        .unwrap();
    checked(g, action);
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
            })
            .unwrap();
        checked(g, a);
    }
    panic!("activation not paid");
}
fn settle_choices(g: &mut Game, target: Option<Target>, accept: bool, pay: bool) {
    g.prepare_triggers();
    for _ in 0..150 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let actions = g.legal_actions();
        let a = if let Some(t) =
            target.filter(|t| actions.contains(&Action::ChooseTarget { target: *t }))
        {
            Action::ChooseTarget { target: t }
        } else if actions.contains(&Action::ChooseOptional { accept }) {
            Action::ChooseOptional { accept }
        } else if !pay && actions.contains(&Action::DeclineResolutionMana) {
            Action::DeclineResolutionMana
        } else if let Some(a) = actions
            .iter()
            .find(|a| matches!(a, Action::PayResolutionMana { .. }))
        {
            a.clone()
        } else if g.state.pending == Pending::Priority {
            Action::Pass
        } else {
            actions
                .iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .or_else(|| actions.iter().find(|a| matches!(a, Action::FinishTargets)))
                .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
                .unwrap()
                .clone()
        };
        checked(g, a);
    }
    panic!("unsettled {:?}", g.state.pending);
}
fn cursed(g: &mut Game, p: Player, bear: CardId) -> CardId {
    let mage = named(g, p, "Spiteful Hexmage", Zone::Hand);
    cast(g, mage, None, &[]);
    settle_choices(g, Some(Target::Card(bear)), true, false);
    g.observe(p)
        .role_timestamps
        .iter()
        .find(|r| r.recipient == g.object_ref(bear))
        .unwrap()
        .aura
        .card
}
fn cleanup_checked(g: &mut Game) {
    let before = g.snapshot();
    let mut branch = g.clone();
    branch.finish_cleanup();
    g.finish_cleanup();
    assert_eq!(g.snapshot(), branch.snapshot());
    for p in [Player::P0, Player::P1] {
        assert_eq!(
            serde_json::to_value(g.observe(p)).unwrap(),
            serde_json::to_value(branch.observe(p)).unwrap()
        );
    }
    let after = g.snapshot();
    g.restore(&before).unwrap();
    g.finish_cleanup();
    assert_eq!(g.snapshot(), after);
}
#[test]
fn base_stat_role_timestamps_and_independent_axes_precede_real_counters_and_growth() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [80; 6];
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        cursed(&mut g, p, bear);
        let counter = named(&mut g, p, "Snakeskin Veil", Zone::Hand);
        cast(&mut g, counter, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (2, 2));
        let wings = named(&mut g, p, "Water Wings", Zone::Hand);
        cast(&mut g, wings, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (5, 5));
        assert!(
            g.keywords(bear)
                .contains(Keywords::FLYING.union(Keywords::HEXPROOF))
        );
        let growth = named(&mut g, p, "Giant Growth", Zone::Hand);
        cast(&mut g, growth, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (8, 8));
        cursed(&mut g, p, bear);
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (5, 5));
        cleanup_checked(&mut g);
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (2, 2));
        assert!(g.observe(p).base_stat_effects.is_empty());
        let mut g = setup(p);
        g.state.players[p.index()].mana = [80; 6];
        let duck = named(&mut g, p, "Turtle-Duck", Zone::Hand);
        cast(&mut g, duck, None, &[]);
        settle(&mut g);
        activate(&mut g, duck);
        settle(&mut g);
        assert_eq!((g.stats(duck).power, g.stats(duck).toughness), (4, 4));
        assert!(g.keywords(duck).contains(Keywords::TRAMPLE));
        cursed(&mut g, p, duck);
        assert_eq!((g.stats(duck).power, g.stats(duck).toughness), (1, 1));
        activate(&mut g, duck);
        settle(&mut g);
        assert_eq!((g.stats(duck).power, g.stats(duck).toughness), (4, 1));
        cleanup_checked(&mut g);
        assert_eq!((g.stats(duck).power, g.stats(duck).toughness), (1, 1));
        assert!(!g.keywords(duck).contains(Keywords::TRAMPLE));
    }
}
#[test]
fn both_perfected_modes_and_all_quandrix_modes_are_complete_and_paid() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..2 {
            let mut g = setup(p);
            let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
            let growth = named(&mut g, p, "Giant Growth", Zone::Hand);
            cast(&mut g, growth, None, &[Target::Card(bear)]);
            settle(&mut g);
            let spell = named(&mut g, p, "Perfected Theory", Zone::Hand);
            cast(&mut g, spell, Some(mode), &[Target::Card(bear)]);
            settle(&mut g);
            assert_eq!(
                (g.stats(bear).power, g.stats(bear).toughness),
                if mode == 0 { (4, 4) } else { (7, 8) }
            );
        }
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let charm = named(&mut g, p, "Quandrix Charm", Zone::Hand);
        cast(&mut g, charm, Some(2), &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (5, 5));
        let age = named(&mut g, p, "A Tale for the Ages", Zone::Hand);
        cast(&mut g, age, None, &[]);
        settle(&mut g);
        let charm = named(&mut g, p, "Quandrix Charm", Zone::Hand);
        cast(&mut g, charm, Some(1), &[Target::Card(age)]);
        settle(&mut g);
        assert_eq!(g.object_view(age).zone, Zone::Graveyard);
        for pay in [false, true] {
            let q = p.other();
            let mut g = setup(q);
            g.state.players[p.index()].mana = [30; 6];
            let bear = named(&mut g, q, "Bear Cub", Zone::Hand);
            cast(&mut g, bear, None, &[]);
            checked(&mut g, Action::Pass);
            let before: u64 = g.state.players[q.index()]
                .mana
                .iter()
                .map(|v| u64::from(*v))
                .sum();
            let charm = named(&mut g, p, "Quandrix Charm", Zone::Hand);
            cast(&mut g, charm, Some(0), &[Target::Spell(bear)]);
            settle_choices(&mut g, None, true, pay);
            assert_eq!(
                g.object_view(bear).zone,
                if pay {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            let after: u64 = g.state.players[q.index()]
                .mana
                .iter()
                .map(|v| u64::from(*v))
                .sum();
            assert_eq!(before - after, if pay { 2 } else { 0 });
        }
    }
}
#[test]
fn zero_base_stats_apply_before_sba_and_reentry_never_inherits_old_effects() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let zero = named(&mut g, p, "Multiply by Zero", Zone::Hand);
        cast(&mut g, zero, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert!(g.observe(p).base_stat_effects.is_empty());
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let counter = named(&mut g, p, "Snakeskin Veil", Zone::Hand);
        cast(&mut g, counter, None, &[Target::Card(bear)]);
        settle(&mut g);
        let zero = named(&mut g, p, "Multiply by Zero", Zone::Hand);
        cast(&mut g, zero, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (1, 1));
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        let old = g.object_ref(bear);
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert!(g.observe(p).base_stat_effects.is_empty());
        cast(&mut g, bear, None, &[]);
        settle(&mut g);
        assert_ne!(old, g.object_ref(bear));
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (2, 2));
        let wing = named(&mut g, p, "Water Wings", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: wing });
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        while g.state.announcing.is_some() {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::FinishTargets))
                .or_else(|| {
                    g.legal_actions()
                        .into_iter()
                        .find(|a| matches!(a, Action::PayCosts { .. }))
                })
                .unwrap();
            checked(&mut g, a);
        }
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert!(g.observe(p).base_stat_effects.is_empty());
        assert_eq!(g.object_view(wing).zone, Zone::Graveyard);
    }
}
#[test]
fn static_setters_keep_entry_timestamps_through_changing_current_conditions() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [80; 6];
        let stag = named(&mut g, p, "Snowmelt Stag", Zone::Hand);
        cast(&mut g, stag, None, &[]);
        settle(&mut g);
        assert_eq!((g.stats(stag).power, g.stats(stag).toughness), (5, 2));
        assert!(g.keywords(stag).contains(Keywords::VIGILANCE));
        activate(&mut g, stag);
        settle(&mut g);
        assert!(g.keywords(stag).contains(Keywords::CANT_BE_BLOCKED));
        let stamp = g.observe(p).base_stat_effects[0].timestamp;
        cleanup_checked(&mut g);
        assert_eq!((g.stats(stag).power, g.stats(stag).toughness), (2, 5));
        assert!(!g.keywords(stag).contains(Keywords::CANT_BE_BLOCKED));
        assert_eq!(g.observe(p).base_stat_effects[0].timestamp, stamp);
        // Isolate the next own-turn condition without rewriting its entry timestamp.
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Main1;
        g.state.pending = Pending::Priority;
        g.state.players[p.index()].mana = [80; 6];
        assert_eq!((g.stats(stag).power, g.stats(stag).toughness), (5, 2));
        cursed(&mut g, p, stag);
        g.state.active = p.other();
        assert_eq!((g.stats(stag).power, g.stats(stag).toughness), (1, 1));
        g.state.active = p;
        assert_eq!((g.stats(stag).power, g.stats(stag).toughness), (1, 1));
        assert_eq!(g.observe(p).base_stat_effects[0].timestamp, stamp);
        let mut g = setup(p);
        g.state.players[p.index()].mana = [80; 6];
        for _ in 0..7 {
            named(&mut g, p, "Forest", Zone::Graveyard);
        }
        named(&mut g, p.other(), "Forest", Zone::Graveyard);
        let doc = named(&mut g, p, "Doc Ock, Sinister Scientist", Zone::Hand);
        cast(&mut g, doc, None, &[]);
        settle(&mut g);
        assert_eq!((g.stats(doc).power, g.stats(doc).toughness), (4, 5));
        assert!(!g.keywords(doc).contains(Keywords::HEXPROOF));
        let curse = cursed(&mut g, p, doc);
        let opt = named(&mut g, p, "Opt", Zone::Hand);
        cast(&mut g, opt, None, &[]);
        settle(&mut g);
        assert_eq!((g.stats(doc).power, g.stats(doc).toughness), (1, 1));
        let remove = named(&mut g, p, "Disenchant", Zone::Hand);
        cast(&mut g, remove, None, &[Target::Card(curse)]);
        settle(&mut g);
        assert_eq!((g.stats(doc).power, g.stats(doc).toughness), (8, 8));
        let villain = named(&mut g, p, "Doc Ock's Henchmen", Zone::Hand);
        cast(&mut g, villain, None, &[]);
        settle(&mut g);
        assert!(g.keywords(doc).contains(Keywords::HEXPROOF));
    }
}
#[test]
fn landfall_optional_base_setting_binds_source_and_both_unblock_consumers_pay_full_bodies() {
    for p in [Player::P0, Player::P1] {
        for accept in [false, true] {
            let mut g = setup(p);
            let elf = named(&mut g, p, "Mirkwood Meditator", Zone::Hand);
            cast(&mut g, elf, None, &[]);
            settle(&mut g);
            let land = named(&mut g, p, "Forest", Zone::Hand);
            checked(&mut g, Action::PlayLand { card: land });
            settle_choices(&mut g, None, accept, false);
            assert_eq!(
                (g.stats(elf).power, g.stats(elf).toughness),
                if accept { (4, 2) } else { (2, 4) }
            );
            cleanup_checked(&mut g);
            assert_eq!((g.stats(elf).power, g.stats(elf).toughness), (2, 4));
        }
        for name in ["Gateway Sneak", "Snooping Page"] {
            let mut g = setup(p);
            let source = named(&mut g, p, name, Zone::Hand);
            cast(&mut g, source, None, &[]);
            settle(&mut g);
            if name == "Gateway Sneak" {
                let gate = named(&mut g, p, "Azorius Guildgate", Zone::Hand);
                checked(&mut g, Action::PlayLand { card: gate });
                settle(&mut g);
            } else {
                let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                let growth = named(&mut g, p, "Giant Growth", Zone::Hand);
                cast(&mut g, growth, None, &[Target::Card(bear)]);
                settle(&mut g);
            }
            assert!(g.keywords(source).contains(Keywords::CANT_BE_BLOCKED));
            g.state.objects[source as usize].summoning_sick = false;
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            g.state.priority = p;
            let drawn = g.state.players[p.index()].cards_drawn_this_turn;
            let life = g.state.players[p.index()].life;
            checked(&mut g, Action::Attack { card: source });
            checked(&mut g, Action::FinishAttackers);
            for _ in 0..50 {
                if g.state.step == Step::Main2 && g.state.stack.is_empty() {
                    break;
                }
                let actions = g.legal_actions();
                let a = actions
                    .into_iter()
                    .find(|a| matches!(a, Action::Pass | Action::FinishBlockers))
                    .unwrap();
                checked(&mut g, a);
            }
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, drawn + 1);
            assert_eq!(
                g.state.players[p.index()].life,
                life - i128::from(name == "Snooping Page")
            );
            cleanup_checked(&mut g);
            assert!(!g.keywords(source).contains(Keywords::CANT_BE_BLOCKED));
        }
    }
}

#[test]
fn primitive_axes_cda_overflow_boundary_and_sparse_owned_heap_are_exact() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        g.state.players[p.index()].mana = [80; 6];
        let body = named(&mut g, p, "Burrowguard Mentor", Zone::Battlefield);
        assert_eq!((g.stats(body).power, g.stats(body).toughness), (1, 1));
        // Isolate the setter primitive; the complete printed callers are paid above.
        let item = g.activation_item(p, body, 0);
        let bytes = g.state_bytes();
        let before = g.snapshot();
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        branch.set_base_stats(&item, Subject::Source, None, Some(5));
        g.set_base_stats(&item, Subject::Source, None, Some(5));
        assert_eq!(g.snapshot(), branch.snapshot());
        assert_eq!(
            g.state_bytes() - bytes,
            std::mem::size_of::<roles::RoleState>() + std::mem::size_of::<BaseStatsEffectView>()
        );
        let after = g.snapshot();
        g.restore(&before).unwrap();
        g.set_base_stats(&item, Subject::Source, None, Some(5));
        assert_eq!(g.snapshot(), after);
        assert_eq!((g.stats(body).power, g.stats(body).toughness), (1, 5));
        g.set_base_stats(&item, Subject::Source, Some(4), None);
        assert_eq!((g.stats(body).power, g.stats(body).toughness), (4, 5));
        cursed(&mut g, p, body);
        assert_eq!((g.stats(body).power, g.stats(body).toughness), (1, 1));
        g.set_base_stats(&item, Subject::Source, None, Some(7));
        assert_eq!((g.stats(body).power, g.stats(body).toughness), (1, 7));
        assert_eq!(
            g.observe(p).base_stat_effects,
            g.observe(p.other()).base_stat_effects
        );
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(body)]);
        settle(&mut g);
        cast(&mut g, body, None, &[]);
        settle(&mut g);
        assert!(g.observe(p).base_stat_effects.is_empty());
        let saved = g.snapshot();
        g.set_base_stats(&item, Subject::Source, Some(9), Some(9));
        assert_eq!(g.snapshot(), saved);
        // Layer 7b precedes checked additive arithmetic: an overwritten printed
        // power must not cause an intermediate overflow before the legal result.
        let item = g.activation_item(p, body, 0);
        g.state.objects[body as usize].growth.power = i128::MAX;
        g.set_base_stats(&item, Subject::Source, Some(-1), None);
        assert_eq!(g.stats(body).power, i128::MAX - 1);
        let artifact = named(&mut g, p, "Mox Jasper", Zone::Battlefield);
        g.state.objects[artifact as usize].counters.plus_one = 3;
        assert_eq!(
            (g.stats(artifact).power, g.stats(artifact).toughness),
            (0, 0)
        );
    }
}

// Insert after Suspect PR322 is merged; requires its completed Presumed/Galuf bodies.
#[test]
fn base_stat_sba_uses_parent_death_exile_before_suspect_return() {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        let mut g = setup(p);
        g.state.players[p.index()].mana = [80; 6];
        // Initial fixture has explicit foreign ownership; every following spell is paid.
        let bear = named(&mut g, q, "Bear Cub", Zone::Battlefield);
        g.state.objects[bear as usize].controller = p;
        let old = g.object_ref(bear);
        let deaths = g.state.creatures_died_this_turn;
        for name in ["Presumed Dead", "Water Wings", "Scorching Dragonfire"] {
            let spell = named(&mut g, p, name, Zone::Hand);
            cast(&mut g, spell, None, &[Target::Card(bear)]);
            settle(&mut g);
        }
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (6, 4));
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(g.object_view(bear).damage, 3);
        let zero = named(&mut g, p, "Multiply by Zero", Zone::Hand);
        cast(&mut g, zero, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
        assert_eq!(g.object_view(bear).owner, q);
        assert_ne!(g.object_ref(bear), old);
        assert_eq!(g.state.creatures_died_this_turn, deaths);
        assert!(!g.is_suspected(bear));
        assert!(g.observe(p).base_stat_effects.is_empty());
        assert!(g.state.temporary_abilities.is_none());
        assert!(g.state.death_exile_replacements.is_none());
        assert!(g.state.stack.is_empty());
        cleanup_checked(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
    }
}
#[test]
fn base_stat_death_lki_feeds_galuf_and_owner_return_clears_old_layers() {
    for p in [Player::P0, Player::P1] {
        let q = p.other();
        let mut g = setup(p);
        g.state.players[p.index()].mana = [80; 6];
        let bear = named(&mut g, q, "Bear Cub", Zone::Battlefield);
        g.state.objects[bear as usize].controller = p;
        let ally = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let old = g.object_ref(bear);
        let deaths = g.state.creatures_died_this_turn;
        for name in ["Presumed Dead", "Galuf's Final Act", "Water Wings"] {
            let spell = named(&mut g, p, name, Zone::Hand);
            cast(&mut g, spell, None, &[Target::Card(bear)]);
            settle(&mut g);
        }
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (7, 4));
        let murder = named(&mut g, p, "Murder", Zone::Hand);
        cast(&mut g, murder, None, &[Target::Card(bear)]);
        settle_choices(&mut g, Some(Target::Card(ally)), true, false);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(
            (g.object_view(bear).owner, g.object_view(bear).controller),
            (q, q)
        );
        assert_ne!(g.object_ref(bear), old);
        assert!(g.is_suspected(bear));
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (2, 2));
        assert_eq!(g.state.objects[ally as usize].counters.plus_one, 7);
        assert_eq!((g.stats(ally).power, g.stats(ally).toughness), (9, 9));
        assert_eq!(g.state.creatures_died_this_turn, deaths + 1);
        assert!(g.observe(p).base_stat_effects.is_empty());
        assert!(!g.keywords(bear).contains(Keywords::FLYING));
        assert!(!g.keywords(bear).contains(Keywords::HEXPROOF));
        assert!(g.state.stack.is_empty());
        cleanup_checked(&mut g);
        assert!(g.is_suspected(bear));
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (2, 2));
    }
}

fn live_equip_target(g: &mut Game, card: CardId, recipient: CardId) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a,Action::AnnounceActivate{card:id,ability:0,..}if *id==card))
        .unwrap();
    checked(g, a);
    checked(
        g,
        Action::ChooseTarget {
            target: Target::Card(recipient),
        },
    );
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::FinishTargets))
            .or_else(|| {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::PayCosts { .. }))
            })
            .unwrap();
        checked(g, a);
    }
    settle(g);
}
#[test]
fn live_static_paid_whole_creatures_aura_and_additive_layers_compose() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let detective = named(&mut g, p, "April, Reporter of the Weird", Zone::Battlefield);
        let enemy = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let archon = named(&mut g, p, "Archon of the Wild Rose", Zone::Hand);
        cast(&mut g, archon, None, &[]);
        settle(&mut g);
        assert_eq!(g.stats(bear).power, 2);
        assert!(!g.keywords(bear).contains(Keywords::FLYING));
        let pacifism = named(&mut g, p, "Pacifism", Zone::Hand);
        cast(&mut g, pacifism, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!((g.stats(bear).power, g.stats(bear).toughness), (4, 4));
        assert!(g.keywords(bear).contains(Keywords::FLYING));
        let maha = named(&mut g, p, "Maha, Its Feathers Night", Zone::Hand);
        cast(&mut g, maha, None, &[]);
        settle(&mut g);
        assert_eq!((g.stats(enemy).power, g.stats(enemy).toughness), (2, 1));
        assert!(
            g.keywords(maha)
                .contains(Keywords::FLYING.union(Keywords::TRAMPLE))
        );
        assert!(
            g.state.objects[maha as usize]
                .card
                .triggers()
                .iter()
                .any(|a| a.condition.event() == TriggerCondition::Ward)
        );
        let burden = named(&mut g, p, "Burden of Proof", Zone::Hand);
        cast(&mut g, burden, None, &[Target::Card(detective)]);
        settle(&mut g);
        // Archon's layer7b applies before Burden's conditional layer7c.
        assert_eq!(
            (g.stats(detective).power, g.stats(detective).toughness),
            (6, 6)
        );
        assert!(g.keywords(detective).contains(Keywords::FLYING));
        for aura in [pacifism, burden] {
            let disenchant = named(&mut g, p, "Disenchant", Zone::Hand);
            cast(&mut g, disenchant, None, &[Target::Card(aura)]);
            settle(&mut g);
        }
        assert_eq!(g.stats(bear).power, 2);
        assert_eq!(g.stats(detective).power, 2);
        assert!(!g.keywords(detective).contains(Keywords::FLYING));
        let bounce = named(&mut g, p, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, None, &[Target::Card(maha)]);
        settle(&mut g);
        assert_eq!(g.stats(enemy).toughness, 2);
    }
}
#[test]
fn live_static_paid_life_equipment_uses_attachment_timestamps_against_cursed() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let first = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let second = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let equipment = named(&mut g, p, "Aettir and Priwen", Zone::Hand);
        cast(&mut g, equipment, None, &[]);
        settle(&mut g);
        live_equip_target(&mut g, equipment, first);
        assert_eq!((g.stats(first).power, g.stats(first).toughness), (20, 20));
        let original = g
            .observe(p)
            .live_static_sources
            .iter()
            .find(|v| v.source.card == equipment)
            .unwrap()
            .timestamp;
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Player(p)]);
        settle(&mut g);
        assert_eq!((g.stats(first).power, g.stats(first).toughness), (18, 18));
        assert_eq!(
            g.observe(p)
                .live_static_sources
                .iter()
                .find(|v| v.source.card == equipment)
                .unwrap()
                .timestamp,
            original
        );
        cursed(&mut g, p, first);
        assert_eq!(g.stats(first).power, 1);
        live_equip_target(&mut g, equipment, first); // Already attached: no new timestamp.
        assert_eq!(g.stats(first).power, 1);
        live_equip_target(&mut g, equipment, second);
        assert_eq!(g.stats(second).power, 18);
        live_equip_target(&mut g, equipment, first);
        assert_eq!((g.stats(first).power, g.stats(first).toughness), (18, 18));
        assert_eq!(g.stats(second).power, 2);
    }
}
