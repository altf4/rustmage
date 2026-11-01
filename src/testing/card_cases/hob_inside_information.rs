//! Focused printed outcomes; shared engine invariants live in core tests.
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn inside_life_payment_insufficient_exact_equal_and_zero_cost_cr118_4() {
    {
        let p = Player::P0;
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
fn inside_life_loss_is_a_cost_and_queues_actual_life_lost_observer() {
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
    {
        let p = Player::P0;
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
fn inside_failed_foreign_entry_rolls_back_paid_life_and_exact_exile_permission_with_retained_mana()
{
    {
        let p = Player::P0;
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
fn inside_optional_kicker_and_live_cost_reduction_modify_mana_only_not_life_value() {
    {
        let p = Player::P0;
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
