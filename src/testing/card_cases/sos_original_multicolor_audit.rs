//! Focused printed outcomes; shared engine invariants live in core tests.
//! Fifteen original non-BG multicolor SOS members: paid, private and owned rules evidence.
use super::*;
fn ck(g: &mut Game, a: Action) {
    big_helpers::checked(g, a)
}
fn cast_paid(g: &mut Game, id: CardId, targets: &[Target]) {
    big_helpers::cast(g, id, None, targets)
}
fn mode(g: &mut Game, id: CardId, mode: u16, targets: &[Target]) {
    big_helpers::cast(g, id, Some(mode), targets)
}
fn pay(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    ck(g, a)
}
fn pause(g: &mut Game) {
    for _ in 0..40 {
        if g.state.pending != Pending::Priority {
            return;
        }
        ck(g, Action::Pass);
    }
    panic!("no decision")
}
fn finish(g: &mut Game) {
    big_helpers::settle(g)
}
fn plain(g: &mut Game) {
    for _ in 0..40 {
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        assert_eq!(g.state.pending, Pending::Priority);
        ck(g, Action::Pass);
    }
    panic!("did not resolve")
}

fn ready_next_main(g: &mut Game, p: Player) {
    big_helpers::next_turn(g);
    big_helpers::next_turn(g);
    for _ in 0..4 {
        ck(g, Action::Pass);
    }
    assert_eq!(g.state.active, p);
    assert_eq!(g.state.step, Step::Main1);
    g.state.players[p.index()].mana = [8; 6];
}
fn attack(g: &mut Game, card: CardId) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    ck(g, Action::Attack { card });
    ck(g, Action::FinishAttackers);
    for _ in 0..8 {
        if g.state.pending == Pending::Blockers {
            return;
        }
        ck(g, Action::Pass);
    }
    panic!("no blockers")
}
fn end_combat(g: &mut Game) {
    for _ in 0..40 {
        if g.state.step == Step::Main2
            && g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
        {
            return;
        }
        if g.state.pending == Pending::Priority {
            ck(g, Action::Pass);
        } else {
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| !matches!(a, Action::Concede))
                .unwrap();
            ck(g, a);
        }
    }
    panic!("combat did not finish")
}
fn xcast(g: &mut Game, spell: CardId, x: u32, target: Option<Target>) {
    ck(g, Action::AnnounceCast { card: spell });
    ck(g, Action::ChooseX { value: x });
    if let Some(target) = target {
        ck(g, Action::ChooseTarget { target });
    }
    pay(g)
}
#[test]
fn sos_audit_multi_cuboid_flash_increment_recheck_and_flying_combat() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        g.state.active = p.other();
        g.state.priority = p.other();
        let source = put(&mut g, p, "Cuboid Colony", Zone::Hand);
        ck(&mut g, Action::Pass);
        cast_paid(&mut g, source, &[]);
        finish(&mut g);
        assert_eq!(stats(&g, source), (1, 1));
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        assert!(
            g.keywords(source).contains(
                Keywords::FLASH
                    .union(Keywords::FLYING)
                    .union(Keywords::TRAMPLE)
            )
        );
        big_helpers::next_turn(&mut g);
        for _ in 0..4 {
            ck(&mut g, Action::Pass);
        }
        g.state.players[p.index()].mana = [8; 6];
        assert_eq!(g.state.active, p);
        let draw = put(&mut g, p, "Quick Study", Zone::Hand);
        cast_paid(&mut g, draw, &[]);
        finish(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        let draw = put(&mut g, p, "Quick Study", Zone::Hand);
        cast_paid(&mut g, draw, &[]);
        let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
        cast_paid(&mut g, growth, &[Target::Card(source)]);
        finish(&mut g);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        assert_eq!(stats(&g, source), (5, 5));
        let blocker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        attack(&mut g, source);
        reject(
            &mut g,
            Action::Block {
                blocker,
                attacker: source,
            },
        );
        ck(&mut g, Action::FinishBlockers);
        end_combat(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 15);
        g.finish_cleanup();
        assert_eq!(stats(&g, source), (2, 2));
    }
}

#[test]
fn sos_audit_multi_pterafractyl_x_entry_life_and_blink_x_zero() {
    {
        let p = Player::P0;
        for x in [0, 3] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Pterafractyl", Zone::Hand);
            xcast(&mut g, source, x, None);
            finish(&mut g);
            assert_eq!(g.state.players[p.index()].life, 22);
            assert_eq!(
                g.object_view(source).zone,
                if x == 0 {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            if x > 0 {
                assert_eq!(g.counter_count(source, CounterKind::PlusOne), u64::from(x));
                assert_eq!(stats(&g, source), (1 + i128::from(x), i128::from(x)));
                assert!(g.keywords(source).contains(Keywords::FLYING));
                let blink = put(&mut g, p, "Daydream", Zone::Hand);
                cast_paid(&mut g, blink, &[Target::Card(source)]);
                finish(&mut g);
                // A fresh incarnation has X=0; Daydream itself supplies exactly one counter.
                assert_eq!(g.object_view(source).zone, Zone::Battlefield);
                assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
                assert_eq!(stats(&g, source), (2, 1));
                assert_eq!(g.state.players[p.index()].life, 24);
            }
        }
    }
}
#[test]
fn sos_audit_multi_quandrix_charm_all_modes_payment_and_base_layer() {
    {
        let p = Player::P0;
        for accept in [false, true] {
            let mut g = scenario(p);
            let draw = put(&mut g, p.other(), "Quick Study", Zone::Hand);
            ck(&mut g, Action::Pass);
            cast_paid(&mut g, draw, &[]);
            g.state.players[p.other().index()].mana = [0, 0, 0, 0, 0, 2];
            ck(&mut g, Action::Pass);
            let charm = put(&mut g, p, "Quandrix Charm", Zone::Hand);
            mode(&mut g, charm, 0, &[Target::Spell(draw)]);
            pause(&mut g);
            assert_eq!(g.actor(), Some(p.other()));
            ck(
                &mut g,
                if accept {
                    Action::PayResolutionMana {
                        payment: [0, 0, 0, 0, 0, 2],
                    }
                } else {
                    Action::DeclineResolutionMana
                },
            );
            finish(&mut g);
            assert_eq!(
                g.state.players[p.other().index()].cards_drawn_this_turn,
                if accept { 2 } else { 0 }
            );
        }
        let mut g = scenario(p);
        let enchant = put(&mut g, p.other(), "Graduation Day", Zone::Battlefield);
        let charm = put(&mut g, p, "Quandrix Charm", Zone::Hand);
        mode(&mut g, charm, 1, &[Target::Card(enchant)]);
        finish(&mut g);
        assert_eq!(g.object_view(enchant).zone, Zone::Graveyard);
        let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let dig = put(&mut g, p, "Dig Site Inventory", Zone::Hand);
        cast_paid(&mut g, dig, &[Target::Card(bear)]);
        finish(&mut g);
        let growth = put(&mut g, p, "Giant Growth", Zone::Hand);
        cast_paid(&mut g, growth, &[Target::Card(bear)]);
        finish(&mut g);
        let charm = put(&mut g, p, "Quandrix Charm", Zone::Hand);
        mode(&mut g, charm, 2, &[Target::Card(bear)]);
        finish(&mut g);
        assert_eq!(stats(&g, bear), (9, 9));
        g.finish_cleanup();
        assert_eq!(stats(&g, bear), (3, 3));
    }
}

#[test]
fn sos_audit_multi_skywhale_opus_x_paid_four_or_five_and_critique_loot() {
    {
        let p = Player::P0;
        for x in [2, 3] {
            let mut g = scenario(p);
            let whale = put(&mut g, p, "Spectacular Skywhale", Zone::Hand);
            cast_paid(&mut g, whale, &[]);
            finish(&mut g);
            assert!(g.keywords(whale).contains(Keywords::FLYING));
            let discard = put(&mut g, p, "Forest", Zone::Hand);
            let spell = put(&mut g, p, "Traumatic Critique", Zone::Hand);
            xcast(&mut g, spell, x, Some(Target::Player(p.other())));
            assert_eq!(
                g.state
                    .stack
                    .iter()
                    .find(|s| s.source.is_some_and(|source| source.id == spell))
                    .unwrap()
                    .cast
                    .unwrap()
                    .mana_spent(),
                u64::from(x) + 2
            );
            pause(&mut g);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 2);
            ck(&mut g, Action::Discard { card: discard });
            finish(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, 20 - i128::from(x));
            assert_eq!(
                g.counter_count(whale, CounterKind::PlusOne),
                if x == 3 { 3 } else { 0 }
            );
            assert_eq!(stats(&g, whale), if x == 3 { (4, 7) } else { (4, 4) });
            g.finish_cleanup();
            assert_eq!(stats(&g, whale), if x == 3 { (4, 7) } else { (1, 4) });
        }
    }
}

#[test]
fn sos_audit_multi_splatter_both_modes_all_creatures_planeswalkers_not_players() {
    {
        let p = Player::P0;
        for which in [0, 1] {
            let mut g = scenario(p);
            let walker = put(&mut g, p, "Vivien Reid", Zone::Hand);
            cast_paid(&mut g, walker, &[]);
            finish(&mut g);
            let a = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let b = put(
                &mut g,
                p.other(),
                "Inkshape Demonstrator",
                Zone::Battlefield,
            );
            let artifact = put(&mut g, p.other(), "Swiftfoot Boots", Zone::Battlefield);
            let spell = put(&mut g, p, "Splatter Technique", Zone::Hand);
            mode(&mut g, spell, which, &[]);
            plain(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                if which == 0 { 4 } else { 0 }
            );
            for id in [a, b] {
                assert_eq!(
                    g.object_view(id).zone,
                    if which == 0 {
                        Zone::Battlefield
                    } else {
                        Zone::Graveyard
                    }
                );
            }
            assert_eq!(
                g.counter_count(walker, CounterKind::Loyalty),
                if which == 0 { 5 } else { 1 }
            );
            assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
            assert_eq!(g.state.players[p.index()].life, 20);
            assert_eq!(g.state.players[p.other().index()].life, 20);
        }
    }
}
#[test]
fn sos_audit_multi_stadium_entry_and_attack_optional_loot() {
    {
        let p = Player::P0;
        for entry in [false, true] {
            let mut g = scenario(p);
            let discard = put(&mut g, p, "Forest", Zone::Hand);
            let source = put(&mut g, p, "Stadium Tidalmage", Zone::Hand);
            cast_paid(&mut g, source, &[]);
            pause(&mut g);
            ck(&mut g, Action::ChooseOptional { accept: entry });
            if entry {
                ck(&mut g, Action::Discard { card: discard });
            }
            finish(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                if entry { 1 } else { 0 }
            );
            assert_eq!(stats(&g, source), (4, 4));
            ready_next_main(&mut g, p);
            let draw_before = g.state.players[p.index()].cards_drawn_this_turn;
            let discard = put(&mut g, p, "Forest", Zone::Hand);
            g.state.step = Step::Attackers;
            g.state.pending = Pending::Attackers;
            ck(&mut g, Action::Attack { card: source });
            ck(&mut g, Action::FinishAttackers);
            pause(&mut g);
            ck(&mut g, Action::ChooseOptional { accept: !entry });
            if !entry {
                ck(&mut g, Action::Discard { card: discard });
            }
            finish(&mut g);
            assert_eq!(
                g.state.players[p.index()].cards_drawn_this_turn,
                draw_before + u64::from(!entry)
            );
        }
    }
}
#[test]
fn sos_audit_multi_snooping_repartee_unblockable_combat_draw_and_life_loss() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Snooping Page", Zone::Hand);
        cast_paid(&mut g, source, &[]);
        finish(&mut g);
        ready_next_main(&mut g, p);
        let spell = put(&mut g, p, "Chase Inspiration", Zone::Hand);
        cast_paid(&mut g, spell, &[Target::Card(source)]);
        finish(&mut g);
        assert!(g.keywords(source).contains(Keywords::CANT_BE_BLOCKED));
        let before = g.state.players[p.index()].cards_drawn_this_turn;
        let blocker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        attack(&mut g, source);
        reject(
            &mut g,
            Action::Block {
                blocker,
                attacker: source,
            },
        );
        ck(&mut g, Action::FinishBlockers);
        end_combat(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, before + 1);
        assert_eq!(g.state.players[p.index()].life, 19);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        g.finish_cleanup();
        assert!(!g.keywords(source).contains(Keywords::CANT_BE_BLOCKED));
    }
}
#[test]
fn sos_audit_multi_relic_sloth_optional_graveyard_and_actual_trample_lifelink() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, "Startled Relic Sloth", Zone::Hand);
        cast_paid(&mut g, source, &[]);
        finish(&mut g);
        ready_next_main(&mut g, p);
        let grave = put(&mut g, p.other(), "Quick Study", Zone::Graveyard);
        g.begin_step(Step::BeginCombat);
        g.give_priority();
        pause(&mut g);
        let mut decline = g.clone();
        ck(&mut decline, Action::FinishTargets);
        finish(&mut decline);
        assert_eq!(decline.object_view(grave).zone, Zone::Graveyard);
        ck(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(grave),
            },
        );
        finish(&mut g);
        assert_eq!(g.object_view(grave).zone, Zone::Exile);
        assert!(
            g.keywords(source)
                .contains(Keywords::TRAMPLE.union(Keywords::LIFELINK))
        );
        let blocker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        attack(&mut g, source);
        ck(
            &mut g,
            Action::Block {
                blocker,
                attacker: source,
            },
        );
        ck(&mut g, Action::FinishBlockers);
        for _ in 0..8 {
            if g.state.pending == Pending::Damage {
                break;
            }
            ck(&mut g, Action::Pass);
        }
        ck(
            &mut g,
            Action::AssignDamage {
                target: Target::Card(blocker),
                amount: 2,
            },
        );
        end_combat(&mut g);
        assert_eq!(g.state.players[p.index()].life, 24);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
    }
}
