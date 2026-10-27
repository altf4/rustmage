//! Paid whole-card and alternative-life casting proofs, with bounded owned roots.
use super::*;
const FIXTURES: &[&str] = &[
    "Inside Information",
    "Forest",
    "Bear Cub",
    "Shock",
    "Stir Up Trouble",
    "Fix What's Broken",
    "Aquatic Alchemist",
    "Harbinger of the Tides",
    "The Master of Lake-town",
    "Negate",
    "Llanowar Elves",
    "Banishing Light",
    "Mox Jasper",
    "Nullpriest of Oblivion",
    "Radagast of Rhosgobel",
    "Bubble Up",
    "Silverquill, the Disputant",
    "Candy Grapple",
    "Witness Protection",
];
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(FIXTURES.contains(&name), "unlisted fixture {name}");
    put(g, p, name, zone)
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}
fn finish(g: &mut Game) {
    for _ in 0..64 {
        if quiet(g) || g.outcome().is_some() {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("inside resolution budget {:?}", g.state.pending);
}
fn bind(g: &mut Game, target: Target) {
    act(g, Action::ChooseTarget { target });
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn announce_life(card: CardId, face: CastFace) -> Action {
    Action::AnnounceExileLife {
        card,
        face,
        flash_surcharge: false,
    }
}
fn clause(g: &mut Game, wanted: CastCostClause) {
    let a = g.state.announcing.as_ref().unwrap();
    let index = a
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|r| !r.paid && r.clause == wanted)
        .expect("cost clause");
    act(
        g,
        Action::ChooseCastCost {
            clause: index as u16,
        },
    );
}
fn expose(g: &mut Game, p: Player, names: &[&str], x: u32) -> (CardId, Vec<CardId>) {
    g.state.players[p.other().index()].library.clear();
    let cards = names
        .iter()
        .map(|name| fixture(g, p.other(), name, Zone::Library))
        .collect::<Vec<_>>();
    let source = fixture(g, p, "Inside Information", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 2, 0, 0, x];
    act(g, Action::AnnounceCast { card: source });
    act(g, Action::ChooseX { value: x });
    bind(g, Target::Player(p.other()));
    pay(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(g.state.stack.last().unwrap().x, Some(x));
    finish(g);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    for &id in &cards {
        assert_eq!(g.object_view(id).zone, Zone::Exile);
    }
    (source, cards)
}
fn cast_life(g: &mut Game, id: CardId, face: CastFace, target: Option<Target>, mv: u32) {
    act(g, announce_life(id, face));
    if let Some(t) = target {
        bind(g, t);
    }
    pay(g);
    clause(g, CastCostClause::AlternativeLife(mv));
}
#[test]
fn inside_paid_xbb_exiles_only_opponent_top_x_publicly_and_life_casts_both_seats() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let hidden = fixture(&mut g, p.other(), "Negate", Zone::Hand);
        let (_, cards) = expose(&mut g, p, &["Bear Cub", "Forest"], 2);
        assert_eq!(g.state.players[p.index()].library.len(), 8);
        for viewer in [p, p.other()] {
            let obs = g.observe(viewer);
            for &id in &cards {
                assert!(
                    obs.objects
                        .iter()
                        .any(|v| v.reference.card == id && v.zone == Zone::Exile)
                );
            }
            if viewer == p {
                assert!(!obs.objects.iter().any(|v| v.reference.card == hidden));
            }
            assert!(obs.objects.iter().all(|v| v.zone != Zone::Library));
        }
        reject(&mut g, Action::AnnounceExile { card: cards[0] });
        cast_life(&mut g, cards[0], CastFace::Normal, None, 2);
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        let item = g.state.stack.last().unwrap();
        assert_eq!(item.controller, p);
        assert_eq!(item.cast.unwrap().origin, Zone::Exile);
        assert_eq!(item.cast.unwrap().method, CastMethod::ExileLife);
        assert_eq!(item.cast.unwrap().payment, [0; 6]);
        finish(&mut g);
        assert_eq!(g.object_view(cards[0]).controller, p);
        assert_eq!(g.object_view(cards[0]).owner, p.other());
        assert_eq!(stats(&g, cards[0]), (2, 2));
    }
}
#[test]
fn inside_zero_x_short_library_and_opponent_target_restriction() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = fixture(&mut g, p, "Inside Information", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 0];
        act(&mut g, Action::AnnounceCast { card: source });
        act(&mut g, Action::ChooseX { value: 0 });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );
        bind(&mut g, Target::Player(p.other()));
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.state.players[p.other().index()].library.len(), 8);
        assert!(g.state.exile_permissions.is_empty());
        let (_, cards) = expose(&mut g, p, &["Shock"], 5);
        assert_eq!(cards.len(), 1);
        assert!(g.state.players[p.other().index()].library.is_empty());
        assert_eq!(g.state.exile_permissions.len(), 1);
    }
}
#[test]
fn inside_land_uses_normal_allowance_control_and_timing_and_expires_this_turn() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (_, cards) = expose(&mut g, p, &["Forest", "Forest", "Bear Cub"], 3);
        g.state.step = Step::BeginCombat;
        reject(&mut g, Action::PlayLand { card: cards[0] });
        reject(&mut g, announce_life(cards[2], CastFace::Normal));
        g.state.step = Step::Main1;
        act(&mut g, Action::PlayLand { card: cards[0] });
        assert_eq!(
            (
                g.object_view(cards[0]).controller,
                g.object_view(cards[0]).owner
            ),
            (p, p.other())
        );
        reject(&mut g, Action::PlayLand { card: cards[1] });
        for _ in 0..96 {
            if g.state.turn > 3 {
                break;
            }
            choose(&mut g, |a| {
                matches!(
                    a,
                    Action::Pass | Action::FinishAttackers | Action::FinishBlockers
                )
            });
        }
        assert!(g.state.turn > 3);
        assert!(g.state.exile_permissions.is_empty());
        assert_eq!(g.object_view(cards[2]).zone, Zone::Exile);
        assert!(!g.has_exile_life_permission(p, cards[2]));
    }
}
#[test]
fn inside_life_payment_insufficient_exact_equal_and_zero_cost_cr118_4() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (_, cards) = expose(&mut g, p, &["Bear Cub"], 1);
        g.state.players[p.index()].life = 1;
        reject(&mut g, announce_life(cards[0], CastFace::Normal));
        g.state.players[p.index()].life = 2;
        assert!(
            g.legal_actions()
                .contains(&announce_life(cards[0], CastFace::Normal))
        );
        cast_life(&mut g, cards[0], CastFace::Normal, None, 2);
        assert_eq!(g.state.players[p.index()].life, 0);
        assert_eq!(g.outcome(), Some(Outcome::Winner(p.other())));
        let mut zero = scenario(p);
        let (_, cards) = expose(&mut zero, p, &["Inside Information"], 1);
        // The exiled X spell has X=0 and fixed mana value two, not the original spell's X.
        cast_life(
            &mut zero,
            cards[0],
            CastFace::Normal,
            Some(Target::Player(p.other())),
            2,
        );
        assert_eq!(zero.state.stack.last().unwrap().x, Some(0));
        assert_eq!(zero.state.stack.last().unwrap().mana_value(), 2);
        finish(&mut zero);
        assert_eq!(zero.state.players[p.index()].life, 18);
        assert_eq!(zero.state.exile_permissions.len(), 0);
        let mut free = scenario(p);
        let (_, cards) = expose(&mut free, p, &["Mox Jasper"], 1);
        free.state.players[p.index()].life = 1;
        cast_life(&mut free, cards[0], CastFace::Normal, None, 0);
        finish(&mut free);
        assert_eq!(free.state.players[p.index()].life, 1);
        assert_eq!(free.object_view(cards[0]).zone, Zone::Battlefield);
    }
}
#[test]
fn inside_adventure_chosen_face_value_and_normal_permission_are_independent() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (_, cards) = expose(&mut g, p, &["Aquatic Alchemist"], 1);
        let id = cards[0];
        let spell = fixture(&mut g, p, "Shock", Zone::Graveyard);
        reject(&mut g, Action::AnnounceAdventure { card: id });
        // Adventure costs three life although the permanent face costs two.
        cast_life(
            &mut g,
            id,
            CastFace::Adventure,
            Some(Target::Card(spell)),
            3,
        );
        assert_eq!(g.state.players[p.index()].life, 17);
        assert_eq!(
            g.state.stack.last().unwrap().source_card,
            Card::from_name("Bubble Up").unwrap()
        );
        finish(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Library);
        assert_eq!(g.object_view(id).zone, Zone::Exile);
        // Adventure grants its own independent normal-face paid permission.
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 1];
        act(&mut g, Action::AnnounceExile { card: id });
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 17);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    }
}
#[test]
fn inside_extra_normal_permission_and_exact_incarnation_do_not_inherit_life_route() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (_, cards) = expose(&mut g, p, &["Bear Cub"], 1);
        let id = cards[0];
        let r = g.object_ref(id);
        g.grant_exile_play_permission(p, r, false);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 1];
        assert!(
            g.legal_actions()
                .contains(&announce_life(id, CastFace::Normal))
        );
        act(&mut g, Action::AnnounceExile { card: id });
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 20);
        g.move_object(id, Zone::Exile);
        assert_ne!(g.object_ref(id), r);
        reject(&mut g, announce_life(id, CastFace::Normal));
        reject(&mut g, Action::AnnounceExile { card: id });
    }
}
#[test]
fn inside_life_loss_is_a_cost_and_queues_actual_life_lost_observer() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (_, cards) = expose(&mut g, p, &["Bear Cub"], 1);
        fixture(
            &mut g,
            p.other(),
            "The Master of Lake-town",
            Zone::Battlefield,
        );
        let before = g.state.players[p.index()].library.len();
        cast_life(&mut g, cards[0], CastFace::Normal, None, 2);
        assert_eq!(g.state.players[p.index()].life_lost_this_turn, 2);
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].library.len(), before - 2);
        assert_eq!(g.object_view(cards[0]).zone, Zone::Battlefield);
    }
}
#[test]
fn inside_mandatory_additional_sacrifice_and_four_mana_alternative_survive() {
    for p in [Player::P0, Player::P1] {
        for mana_route in [false, true] {
            let mut g = scenario(p);
            let (_, cards) = expose(&mut g, p, &["Stir Up Trouble"], 1);
            let target = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let fodder = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
            g.state.players[p.index()].mana = if mana_route {
                [0, 0, 0, 0, 0, 4]
            } else {
                [0; 6]
            };
            act(&mut g, announce_life(cards[0], CastFace::Normal));
            act(
                &mut g,
                Action::ChooseSpellCost {
                    cost: u16::from(mana_route),
                },
            );
            bind(&mut g, Target::Card(target));
            pay(&mut g);
            clause(&mut g, CastCostClause::AlternativeLife(1));
            if !mana_route {
                clause(&mut g, CastCostClause::Additional);
                let object = g.object_ref(fodder);
                act(&mut g, Action::ChooseAdditionalCost { object });
            }
            finish(&mut g);
            assert_eq!(g.object_view(target).zone, Zone::Graveyard);
            assert_eq!(
                g.object_view(fodder).zone,
                if mana_route {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(g.state.players[p.index()].life, 19);
        }
    }
}
#[test]
fn inside_additional_x_life_and_alternative_life_remain_distinct_even_equal() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (_, cards) = expose(&mut g, p, &["Fix What's Broken"], 1);
        let mv = g.object_view(cards[0]).card.fixed_mana_value() as u32;
        g.state.players[p.index()].life = i128::from(mv * 2 - 1);
        act(&mut g, announce_life(cards[0], CastFace::Normal));
        reject(&mut g, Action::ChooseX { value: mv });
        act(&mut g, Action::ChooseX { value: 0 });
        pay(&mut g);
        clause(&mut g, CastCostClause::Life(0));
        clause(&mut g, CastCostClause::AlternativeLife(mv));
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, i128::from(mv - 1));
        let mut equal = scenario(p);
        let (_, cards) = expose(&mut equal, p, &["Fix What's Broken"], 1);
        act(&mut equal, announce_life(cards[0], CastFace::Normal));
        act(&mut equal, Action::ChooseX { value: mv });
        pay(&mut equal);
        clause(&mut equal, CastCostClause::Life(mv));
        clause(&mut equal, CastCostClause::AlternativeLife(mv));
        finish(&mut equal);
        assert_eq!(equal.state.players[p.index()].life, 20 - i128::from(2 * mv));
    }
}
#[test]
fn inside_flash_surcharge_is_additional_mana_and_timing_stays_normal() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (_, cards) = expose(&mut g, p, &["Harbinger of the Tides"], 1);
        g.state.step = Step::BeginCombat;
        reject(&mut g, announce_life(cards[0], CastFace::Normal));
        let action = Action::AnnounceExileLife {
            card: cards[0],
            face: CastFace::Normal,
            flash_surcharge: true,
        };
        reject(&mut g, action.clone());
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        act(&mut g, action);
        pay(&mut g);
        clause(&mut g, CastCostClause::AlternativeLife(2));
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 18);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert_eq!(g.object_view(cards[0]).zone, Zone::Battlefield);
    }
}
#[test]
fn inside_owned_life_payment_roots_raw_index_restore_and_wrong_actor_rollback() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (_, cards) = expose(&mut g, p, &["Bear Cub"], 1);
        act(&mut g, announce_life(cards[0], CastFace::Normal));
        pay(&mut g);
        super::contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        let action = raw
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::ChooseCastCost { .. }))
            .unwrap();
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        act(&mut raw, action.clone());
        indexed.apply_index(p, indexed.revision(), index).unwrap();
        assert_eq!(raw.snapshot(), indexed.snapshot());
        indexed.restore(&saved).unwrap();
        assert_eq!(indexed.state.players[p.index()].life, 20);
        act(&mut indexed, action);
        finish(&mut indexed);
        finish(&mut raw);
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.state.players[p.index()].life, 18);
    }
}
#[test]
fn inside_actual_counterspell_does_not_refund_life_or_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let (_, cards) = expose(&mut g, p, &["Shock"], 1);
        cast_life(
            &mut g,
            cards[0],
            CastFace::Normal,
            Some(Target::Player(p.other())),
            1,
        );
        act(&mut g, Action::Pass);
        let counter = fixture(&mut g, p.other(), "Negate", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 1];
        cast(&mut g, counter, Some(Target::Spell(cards[0])));
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].life, 19);
        assert_eq!(g.state.players[p.other().index()].life, 20);
        assert_eq!(g.object_view(cards[0]).zone, Zone::Graveyard);
        assert!(!g.has_exile_life_permission(p, cards[0]));
    }
}
#[test]
fn inside_failed_foreign_entry_rolls_back_paid_life_and_exact_exile_permission_with_retained_mana()
{
    for p in [Player::P0, Player::P1] {
        for keep in [false, true] {
            let mut g = scenario(p);
            let (_, cards) = expose(&mut g, p, &["Candy Grapple"], 1);
            let spell = cards[0];
            let silver = fixture(&mut g, p, "Silverquill, the Disputant", Zone::Battlefield);
            g.state.objects[silver as usize].growth.power = -1;
            let enemy = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let aura = fixture(&mut g, p.other(), "Witness Protection", Zone::Battlefield);
            g.state.objects[aura as usize].attached_to = Some(g.object_ref(enemy));
            let prison = fixture(&mut g, p, "Banishing Light", Zone::Hand);
            g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 2];
            cast(&mut g, prison, None);
            for _ in 0..24 {
                if g.state.pending != Pending::Priority {
                    break;
                }
                act(&mut g, Action::Pass);
            }
            bind(&mut g, Target::Card(aura));
            finish(&mut g);
            assert_eq!(g.object_view(aura).zone, Zone::Exile);
            let forest = fixture(&mut g, p, "Forest", Zone::Battlefield);
            let original = g.object_ref(spell);
            act(&mut g, announce_life(spell, CastFace::Normal));
            let sacrifice = Some(g.object_ref(prison));
            act(&mut g, Action::ChooseBargain { sacrifice });
            act(&mut g, Action::ChooseCasualty { pay: true });
            bind(&mut g, Target::Card(enemy));
            act(&mut g, Action::TapMana { card: forest });
            pay(&mut g);
            clause(&mut g, CastCostClause::AlternativeLife(2));
            assert_eq!(g.state.players[p.index()].life, 18);
            clause(&mut g, CastCostClause::Additional);
            assert_eq!(g.actor(), Some(p.other()));
            super::contracts::branch_contract(g.clone());
            let object = g.object_ref(silver);
            act(&mut g, Action::ChooseAuraAttachment { object });
            assert!(matches!(g.state.pending, Pending::ManaRollback { .. }));
            act(&mut g, Action::ChooseManaRollback { keep });
            assert_eq!(g.state.players[p.index()].life, 20);
            assert_eq!(g.object_ref(spell), original);
            assert_eq!(g.object_view(spell).zone, Zone::Exile);
            assert!(g.has_exile_life_permission(p, spell));
            assert_eq!(g.object_view(prison).zone, Zone::Battlefield);
            assert_eq!(g.object_view(aura).zone, Zone::Exile);
            assert_eq!(g.object_view(forest).tapped, keep);
            assert_eq!(
                g.state.players[p.index()].mana,
                [0, 0, 0, 0, u32::from(keep), 0]
            );
        }
    }
}
#[test]
fn inside_only_top_x_leave_larger_library_and_owner_gets_no_play_permission() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let bottom = fixture(&mut g, p.other(), "Negate", Zone::Library);
        let middle = fixture(&mut g, p.other(), "Forest", Zone::Library);
        let top = fixture(&mut g, p.other(), "Bear Cub", Zone::Library);
        let source = fixture(&mut g, p, "Inside Information", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 2];
        act(&mut g, Action::AnnounceCast { card: source });
        act(&mut g, Action::ChooseX { value: 2 });
        bind(&mut g, Target::Player(p.other()));
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(bottom).zone, Zone::Library);
        assert_eq!(
            g.state.players[p.other().index()].library.last(),
            Some(&bottom)
        );
        assert_eq!(g.state.players[p.other().index()].library.len(), 9);
        for id in [middle, top] {
            assert_eq!(g.object_view(id).zone, Zone::Exile);
            assert!(g.has_exile_life_permission(p, id));
            assert!(!g.has_exile_life_permission(p.other(), id));
        }
        for viewer in [p, p.other()] {
            let obs = g.observe(viewer);
            assert!(!obs.objects.iter().any(|v| v.id == bottom));
            assert_eq!(obs.exile_permissions.len(), 2);
            assert!(obs.exile_permissions.iter().all(|r| r.life_instead_of_mana));
        }
        let life_row = serde_json::to_value(g.state.exile_permissions[0]).unwrap();
        assert_eq!(life_row["life_instead_of_mana"], true);
        g.grant_exile_play_permission(p, g.object_ref(top), false);
        let ordinary = g
            .state
            .exile_permissions
            .iter()
            .find(|r| !r.life_instead_of_mana)
            .unwrap();
        assert!(
            serde_json::to_value(ordinary)
                .unwrap()
                .get("life_instead_of_mana")
                .is_none()
        );
        act(&mut g, Action::Pass);
        assert_eq!(g.actor(), Some(p.other()));
        reject(&mut g, announce_life(top, CastFace::Normal));
        reject(&mut g, Action::PlayLand { card: middle });
    }
}
#[test]
fn inside_optional_kicker_and_live_cost_reduction_modify_mana_only_not_life_value() {
    for p in [Player::P0, Player::P1] {
        for reduced in [false, true] {
            let mut g = scenario(p);
            let (_, cards) = expose(&mut g, p, &["Nullpriest of Oblivion"], 1);
            if reduced {
                fixture(&mut g, p, "Radagast of Rhosgobel", Zone::Battlefield);
            }
            let returned = fixture(&mut g, p, "Bear Cub", Zone::Graveyard);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, if reduced { 1 } else { 3 }];
            act(&mut g, announce_life(cards[0], CastFace::Normal));
            act(&mut g, Action::ChooseKicker { pay: true });
            pay(&mut g);
            clause(&mut g, CastCostClause::AlternativeLife(2));
            assert_eq!(g.state.players[p.index()].life, 18);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            for _ in 0..24 {
                if matches!(g.state.pending, Pending::Targets { .. }) {
                    break;
                }
                choose(&mut g, |a| {
                    matches!(a, Action::Pass | Action::PutTrigger { .. })
                });
            }
            bind(&mut g, Target::Card(returned));
            finish(&mut g);
            assert_eq!(g.object_view(returned).zone, Zone::Battlefield);
            assert_eq!(g.object_view(returned).controller, p);
            assert_eq!(g.object_view(cards[0]).zone, Zone::Battlefield);
        }
    }
}
#[test]
fn inside_chosen_life_permission_does_not_inherit_other_routes_modifiers() {
    for p in [Player::P0, Player::P1] {
        for life_route in [false, true] {
            let mut g = scenario(p);
            let (_, cards) = expose(&mut g, p, &["Shock"], 1);
            let id = cards[0];
            // An independently granted permission can carry its own "this way"
            // replacement and any-color mana, without changing the life route.
            g.grant_exile_play_permission(p, g.object_ref(id), false);
            let other = g
                .state
                .exile_permissions
                .iter_mut()
                .find(|r| !r.life_instead_of_mana)
                .unwrap();
            other.spend_any_mana = true;
            other.exile_on_graveyard = true;
            if life_route {
                cast_life(
                    &mut g,
                    id,
                    CastFace::Normal,
                    Some(Target::Player(p.other())),
                    1,
                );
            } else {
                g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
                act(&mut g, Action::AnnounceExile { card: id });
                bind(&mut g, Target::Player(p.other()));
                pay(&mut g);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            }
            let cast = g.state.stack.last().unwrap().cast.unwrap();
            assert_eq!(cast.exile_on_graveyard, !life_route);
            assert_eq!(cast.spend_any_mana, !life_route);
            assert!(!cast.stash_origin);
            finish(&mut g);
            assert_eq!(
                g.object_view(id).zone,
                if life_route {
                    Zone::Graveyard
                } else {
                    Zone::Exile
                }
            );
            assert_eq!(g.state.players[p.other().index()].life, 18);
            assert_eq!(
                g.state.players[p.index()].life,
                if life_route { 19 } else { 20 }
            );
        }
    }
}
