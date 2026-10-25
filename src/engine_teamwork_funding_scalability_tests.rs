//! Coupled paid Teamwork/Mana search regressions. The scoped node budget is
//! disabled by default and reset by RAII, including panic unwinding; it prevents
//! the intentionally red development check from becoming a runaway workload.
use super::*;
use crate::engine::assurance::big_helpers::{cast, checked, named, settle, setup};
use std::cell::Cell;
#[derive(Clone, Copy)]
struct Limit {
    nodes: u64,
    maximum: u64,
}
thread_local! { static LIMIT: Cell<Option<Limit>> = const { Cell::new(None) }; }
struct Budget;
impl Budget {
    fn new(maximum: u64) -> Self {
        LIMIT.with(|c| {
            assert!(c.get().is_none());
            c.set(Some(Limit { nodes: 0, maximum }));
        });
        Self
    }
    fn nodes(&self) -> u64 {
        LIMIT.with(|c| c.get().unwrap().nodes)
    }
}
impl Drop for Budget {
    fn drop(&mut self) {
        LIMIT.with(|c| c.set(None));
    }
}
pub(super) fn visit_teamwork_node() {
    LIMIT.with(|c| {
        if let Some(mut limit) = c.get() {
            limit.nodes += 1;
            c.set(Some(limit));
            assert!(
                limit.nodes <= limit.maximum,
                "coupled Teamwork/Mana search exceeded scoped {}-node budget before mana readiness",
                limit.maximum
            );
        }
    });
}
fn reject(g: &mut Game, action: Action) {
    let snapshot = g.snapshot();
    let revision = g.revision();
    let views = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), snapshot);
    assert_eq!(g.revision(), revision);
    assert_eq!(
        [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap()),
        views
    );
}
fn clause(g: &mut Game, kind: CastCostClause) {
    let clause = g
        .state
        .announcing
        .as_ref()
        .unwrap()
        .cast_costs
        .as_ref()
        .unwrap()
        .clauses
        .iter()
        .position(|r| r.clause == kind)
        .unwrap() as u16;
    checked(g, Action::ChooseCastCost { clause });
}
fn paid_iron(g: &mut Game, p: Player) -> CardId {
    let id = named(g, p, "Ironheart, Clever Champion", Zone::Hand);
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 4];
    cast(g, id, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle(g);
    id
}
#[test]
fn msh_teamwork_funding_scalability_paid_atlantis_zero_floated_blue_then_real_islands_both_orders()
{
    for p in [Player::P0, Player::P1] {
        for mana_first in [false, true] {
            let mut g = setup(p);
            let iron = paid_iron(&mut g, p);
            // Initial board of forty ordinary existing Doombot tokens: reachable by
            // repeated token effects, not forty physical copies in a legal deck.
            g.create_tokens(p, Card::MshDoombotToken, 40, false);
            settle(&mut g);
            let robots: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter_map(|(i, o)| {
                    (o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.card == Card::MshDoombotToken)
                        .then_some(i as CardId)
                })
                .collect();
            assert_eq!(robots.len(), 40);
            let islands = [
                named(&mut g, p, "Island", Zone::Battlefield),
                named(&mut g, p, "Island", Zone::Battlefield),
            ];
            let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let spell = named(&mut g, p, "Atlantis Attacks", Zone::Hand);
            g.state.players[p.index()].mana = [0; 6];
            let budget = Budget::new(256);
            checked(&mut g, Action::AnnounceCast { card: spell });
            assert!(matches!(g.state.pending, Pending::TeamworkIntent { .. }));
            let actions = g.legal_actions();
            assert!(actions.contains(&Action::ChooseTeamwork { pay: false }));
            assert!(actions.contains(&Action::ChooseTeamwork { pay: true }));
            println!(
                "coupled paid Atlantis actor={p:?} mana_first={mana_first} nodes={}",
                budget.nodes()
            );
            drop(budget);
            g = orphan(g, Action::ChooseTeamwork { pay: true });
            checked(&mut g, Action::ChooseMode { mode: 2 });
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p),
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                checked(&mut g, Action::FinishTargets);
            }
            for card in islands {
                checked(&mut g, Action::TapMana { card });
            }
            assert_eq!(g.state.players[p.index()].mana, [0, 2, 0, 0, 0, 0]);
            checked(&mut g, Action::BeginImprovise);
            let (teammates, helpers) = if mana_first {
                (&robots[5..7], &robots[..5])
            } else {
                (&robots[..2], &robots[2..7])
            };
            if !mana_first {
                clause(&mut g, CastCostClause::Teamwork(4));
                for &id in teammates {
                    let object = g.object_ref(id);
                    checked(&mut g, Action::ChooseTeammate { object });
                }
                checked(&mut g, Action::FinishTeamwork);
            }
            clause(&mut g, CastCostClause::Mana);
            if !mana_first {
                let object = g.object_ref(teammates[0]);
                reject(&mut g, Action::ChooseImproviser { object });
            }
            for &id in helpers {
                let object = g.object_ref(id);
                checked(&mut g, Action::ChooseImproviser { object });
            }
            let pay = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            checked(&mut g, pay);
            if mana_first {
                clause(&mut g, CastCostClause::Teamwork(4));
                let object = g.object_ref(helpers[0]);
                reject(&mut g, Action::ChooseTeammate { object });
                for &id in teammates {
                    let object = g.object_ref(id);
                    checked(&mut g, Action::ChooseTeammate { object });
                }
                checked(&mut g, Action::FinishTeamwork);
            }
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(
                teammates
                    .iter()
                    .chain(helpers)
                    .all(|&id| g.object_view(id).tapped)
            );
            assert!(!g.object_view(iron).tapped);
            assert_eq!(
                g.state.stack.last().unwrap().cast.unwrap().teamwork,
                Some(true)
            );
            settle(&mut g);
            assert_eq!(g.object_view(victim).zone, Zone::Hand);
            assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
            assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
                && o.controller == p
                && o.card == Card::MshLeviathanToken));
        }
    }
}

fn orphan(g: Game, action: Action) -> Game {
    let actor = g.actor().unwrap();
    let revision = g.revision();
    let before = g.snapshot();
    let seen = [Player::P0, Player::P1].map(|p| serde_json::to_value(g.observe(p)).unwrap());
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let (mut raw, mut indexed) = (g.clone(), g.clone());
    drop(g);
    raw.apply(actor, action.clone()).unwrap();
    indexed.apply_index(actor, revision, index).unwrap();
    let after = raw.snapshot();
    assert_eq!(after, indexed.snapshot());
    assert_eq!(
        [Player::P0, Player::P1].map(|p| serde_json::to_value(raw.observe(p)).unwrap()),
        [Player::P0, Player::P1].map(|p| serde_json::to_value(indexed.observe(p)).unwrap())
    );
    raw.restore(&before).unwrap();
    let mut restored = seen;
    for v in &mut restored {
        v["revision"] = serde_json::json!(raw.revision());
    }
    assert_eq!(
        [Player::P0, Player::P1].map(|p| serde_json::to_value(raw.observe(p)).unwrap()),
        restored
    );
    assert_eq!(
        raw.apply_index(actor, revision, index),
        Err(Error::StaleRevision)
    );
    raw.apply(actor, action).unwrap();
    assert_eq!(raw.snapshot(), after);
    indexed
}
fn context(g: &Game, p: Player, spell: CardId) -> Box<CostContext> {
    Box::new(CostContext {
        actor: p,
        source: g.object_ref(spell),
        clauses: vec![
            CastCostRow {
                clause: CastCostClause::Teamwork(4),
                paid: false,
            },
            CastCostRow {
                clause: CastCostClause::Mana,
                paid: false,
            },
        ],
        active: None,
        casualty: vec![],
        additional: None,
        forage: None,
        selected: vec![],
        exact: false,
        counters: vec![],
        mana: Some(ManaCost {
            colored: [0, 2, 0, 0, 0, 0],
            generic: 5,
        }),
        teammates: vec![],
        teamwork_exact: false,
    })
}
fn world<'g>(g: &'g Game, p: Player, spell: CardId, c: Box<CostContext>) -> FundingState<'g> {
    let goal = FundingGoal {
        actor: p,
        required: [0, 2, 0, 0, 0, 0],
        generic: 5,
        source: None,
        reserve: None,
    };
    let mut w = FundingState::from_game(g, &goal);
    w.set_purpose(g.spell_mana_purpose_for_card(
        p,
        spell,
        Card::from_name("Atlantis Attacks").unwrap(),
    ));
    w.install_cost_context(Some(c));
    w
}
#[test]
fn msh_teamwork_funding_scalability_small_independent_shared_resource_oracle_and_active_orders() {
    // The diagnostic itself must not leak a counter into other tests on unwind.
    assert!(
        std::panic::catch_unwind(|| {
            let _budget = Budget::new(0);
            visit_teamwork_node();
        })
        .is_err()
    );
    assert!(LIMIT.with(|c| c.get().is_none()));
    visit_teamwork_node();
    for p in [Player::P0, Player::P1] {
        for robots in 0u32..=7 {
            for blue in 0u32..=3 {
                for active in [None, Some(0), Some(1)] {
                    let mut g = setup(p);
                    paid_iron(&mut g, p);
                    g.create_tokens(p, Card::MshDoombotToken, robots.into(), false);
                    settle(&mut g);
                    let spell = named(&mut g, p, "Atlantis Attacks", Zone::Hand);
                    g.state.players[p.index()].mana = [0, blue, 0, 0, 0, 0];
                    let mut c = context(&g, p, spell);
                    c.active = active;
                    let w = world(&g, p, spell, c);
                    let before = g.snapshot();
                    // Every helper has power3 and is an artifact: at least two pay Teamwork,
                    // and each of the remaining helpers can pay only one generic mana.
                    let n = robots + 1;
                    let expected = blue >= 2 && n >= 2 && 5 <= n - 2 + blue - 2;
                    let budget = Budget::new(4096);
                    assert_eq!(
                        w.cost_completion(),
                        expected,
                        "robots={robots} blue={blue} active={active:?}"
                    );
                    drop(budget);
                    assert_eq!(g.snapshot(), before);
                }
            }
        }
    }
}
#[test]
fn msh_teamwork_funding_scalability_paid_unfundable_second_blue_is_atomic() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        paid_iron(&mut g, p);
        g.create_tokens(p, Card::MshDoombotToken, 40, false);
        settle(&mut g);
        named(&mut g, p, "Island", Zone::Battlefield);
        let spell = named(&mut g, p, "Atlantis Attacks", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5];
        let budget = Budget::new(256);
        reject(&mut g, Action::AnnounceCast { card: spell });
        assert_eq!(g.object_view(spell).zone, Zone::Hand);
        drop(budget);
    }
}
#[test]
fn msh_teamwork_funding_scalability_provider_control_suppression_and_missing_colors() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = setup(p);
            let iron = paid_iron(&mut g, p);
            if mode == 1 {
                // Explicit earlier-layer provider-control fixture.
                g.change_controller(g.object_ref(iron), p.other());
            }
            if mode == 2 {
                let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
                g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
                cast(&mut g, aura, None, &[Target::Card(iron)]);
                settle(&mut g);
            }
            g.create_tokens(p, Card::MshDoombotToken, 20, false);
            settle(&mut g);
            let spell = named(&mut g, p, "Atlantis Attacks", Zone::Hand);
            assert!(improvise::potential(
                &g,
                Card::from_name("Atlantis Attacks").unwrap()
            ));
            for blue in 0..=1 {
                g.state.players[p.index()].mana = [0, blue, 0, 0, 0, 0];
                let c = context(&g, p, spell);
                let w = world(&g, p, spell, c);
                let budget = Budget::new(256);
                assert!(!w.cost_completion());
                assert_eq!(budget.nodes(), 0);
                drop(budget);
            }
            g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 5];
            let c = context(&g, p, spell);
            let w = world(&g, p, spell, c);
            assert!(w.cost_completion());
        }
    }
}
#[test]
fn msh_teamwork_funding_scalability_signed_exact_prefix_and_live_tap_quantity() {
    use crate::cards::{CountFilter, CountZone, StatValue};
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let iron = paid_iron(&mut g, p);
        g.create_tokens(p, Card::MshDoombotToken, 8, false);
        settle(&mut g);
        let robots: Vec<_> = g
            .state
            .objects
            .iter()
            .enumerate()
            .filter_map(|(i, o)| {
                (o.zone == Zone::Battlefield && o.card == Card::MshDoombotToken)
                    .then_some(i as CardId)
            })
            .collect();
        let negative = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.objects[negative as usize].growth.power = -5;
        let spell = named(&mut g, p, "Atlantis Attacks", Zone::Hand);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 0];
        let mut c = context(&g, p, spell);
        c.teammates = vec![g.object_ref(negative)];
        c.teamwork_exact = true;
        let w = world(&g, p, spell, c.clone());
        assert!(!w.cost_completion());
        c.teamwork_exact = false;
        let w = world(&g, p, spell, c.clone());
        assert!(w.cost_completion());
        c.teammates.extend([
            g.object_ref(iron),
            g.object_ref(robots[0]),
            g.object_ref(robots[1]),
        ]);
        c.teamwork_exact = true;
        let mut w = world(&g, p, spell, c);
        assert!(w.cost_completion());
        // Bounded earlier-layer tap-count primitive (not an invented card/CDA).
        // It stays live in the exact borrowed world rather than being frozen by
        // the feasibility preflight; signed teammate powers also stay distinct.
        let value = StatValue::TappedCount(CountFilter {
            zone: CountZone::Battlefield,
            relation: 0,
            kind: 1,
            exclude: 0,
            subtype: None,
            basic: false,
            other: false,
            cards_only: false,
        });
        assert_eq!(w.value(value, iron), 0);
        let index = w.resources.iter().position(|r| r.id == robots[0]).unwrap();
        w.resources[index].object.tapped = true;
        assert_eq!(w.value(value, iron), 1);
        assert_eq!(w.power(negative), -3);
    }
}
#[test]
fn msh_teamwork_funding_scalability_general_sacrifice_restores_improvise_before_mana() {
    // Private generic-cost boundary, not a new printed producer: this ordinary
    // enchantment sacrifice must remain available before the mana clause.
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let iron = paid_iron(&mut g, p);
        let aura = named(&mut g, p, "Witness Protection", Zone::Hand);
        g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 0];
        cast(&mut g, aura, None, &[Target::Card(iron)]);
        settle(&mut g);
        g.create_tokens(p, Card::MshDoombotToken, 8, false);
        settle(&mut g);
        let spell = named(&mut g, p, "Atlantis Attacks", Zone::Hand);
        g.state.players[p.index()].mana = [0, 2, 0, 0, 0, 0];
        let mut c = context(&g, p, spell);
        c.additional = Some(AdditionalCost {
            kind: EffectChoiceKind::Sacrifice,
            filter: SacrificeFilter {
                keywords: Keywords::NONE,
                source_only: false,
                kinds: 32,
                nonland: false,
                other: false,
                tokens: None,
                artifact_enchantment_or_token: false,
                subtype: None,
            },
        });
        c.clauses.insert(
            0,
            CastCostRow {
                clause: CastCostClause::Additional,
                paid: false,
            },
        );
        let w = world(&g, p, spell, c);
        assert!(!w.mana_clause_payable(w.cast_costs.as_ref().unwrap()));
        let before = g.snapshot();
        assert!(w.cost_completion());
        assert_eq!(g.snapshot(), before);
        assert_eq!(g.object_view(aura).zone, Zone::Battlefield);
        assert!(!improvise::available(
            &g,
            p,
            Card::from_name("Atlantis Attacks").unwrap()
        ));
    }
}

thread_local! { static FUNDING_LIMIT: Cell<Option<Limit>> = const { Cell::new(None) }; }
struct FundingBudget;
impl FundingBudget {
    fn new(maximum: u64) -> Self {
        FUNDING_LIMIT.with(|c| {
            assert!(c.get().is_none());
            c.set(Some(Limit { nodes: 0, maximum }));
        });
        Self
    }
}
impl Drop for FundingBudget {
    fn drop(&mut self) {
        FUNDING_LIMIT.with(|c| c.set(None));
    }
}
pub(super) fn visit_funding_node() {
    FUNDING_LIMIT.with(|c| {
        if let Some(mut limit) = c.get() {
            limit.nodes += 1;
            c.set(Some(limit));
            assert!(
                limit.nodes <= limit.maximum,
                "exact diagnostic exceeded scoped {} funding-node budget",
                limit.maximum
            );
        }
    });
}
#[test]
#[ignore = "Local exact original config14 diagnostic; requires a cache output path and a cheap scoped search budget"]
fn msh_teamwork_config14_exact_context_diagnostic() {
    let output =
        std::path::PathBuf::from(std::env::var("RUSTMAGE_TEAMWORK_CONTEXT_RECORD").unwrap());
    assert!(
        output.starts_with(std::path::PathBuf::from(std::env::var("HOME").unwrap()).join(".cache"))
    );
    assert!(!output.exists());
    let pool: serde_json::Value =
        serde_json::from_str(include_str!("../data/assurance/msh-broad-legal-games.json")).unwrap();
    let case = &pool["games"][14];
    assert_eq!(case["seed"].as_u64(), Some(5571876328824635697));
    let decks = std::array::from_fn(|i| {
        let deck = pool["decks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["id"] == case["decks"][i])
            .unwrap();
        deck["cards"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|row| {
                std::iter::repeat_n(
                    Card::from_name(row[0].as_str().unwrap()).unwrap(),
                    row[1].as_u64().unwrap() as usize,
                )
            })
            .collect()
    });
    let config = Config {
        decks,
        seed: case["seed"].as_u64().unwrap(),
        starting_player: serde_json::from_value(case["starting_player"].clone()).unwrap(),
        deck_rules: DeckRules::Standard,
    };
    let mut g = Game::new(config).unwrap();
    let mut ledger = Vec::new();
    for _ in 0..1141 {
        assert!(g.outcome().is_none());
        let actor = g.actor().unwrap();
        let actions = g.legal_actions();
        let index = crate::policy::smoke_index(&g, &actions);
        let action = actions[index].clone();
        ledger.push((actor, action.clone()));
        g.apply(actor, action).unwrap();
    }
    assert_eq!(g.state.turn, 31);
    assert_eq!(g.state.objects.len(), 128);
    assert!(matches!(
        g.state.pending,
        Pending::TeamworkIntent { player: Player::P0 }
    ));
    assert_eq!(g.state.announcing.as_ref().unwrap().item.object.card, 59);
    let mut contexts = Vec::new();
    for pay in [false, true] {
        let mut a = g.state.announcing.as_ref().unwrap().clone();
        a.item.cast.as_mut().unwrap().teamwork = Some(pay);
        a.initialize_cast_costs(improvise::potential(&g, a.item.source_card));
        for cost in a.cost_options() {
            let mut choice = a.clone();
            choice.cost = Some(cost);
            let goal = g.funding_goal(&choice);
            let mut w = FundingState::from_game_with_convoke(
                &g,
                &goal,
                choice.item.source_card.has_convoke(),
            );
            w.set_purpose(g.purpose_for_item(&choice.item));
            w.set_cost_context(Some(&choice));
            let resources: Vec<_> = w.resources.iter().filter(|r| r.object.zone == Zone::Battlefield).map(|r| serde_json::json!({"id":r.id,"definition":r.object.card.identity(),"name":w.name(r.id),"controller":r.object.controller,"tapped":r.object.tapped,"kinds":w.kinds(r.id),"colors":w.colors(r.id),"power":w.power(r.id),"mana_activations":format!("{:?}", w.mana_activations(r.id).map(|(i,a,_)|(i,a)).collect::<Vec<_>>())})).collect();
            let c = w.cast_costs.as_ref();
            contexts.push(serde_json::json!({"pay_teamwork":pay,"goal":format!("{goal:?}"),"pool":w.mana,"lots":format!("{:?}",w.lots),"available_mana":mana_lots::available(w.mana,&w.lots,w.purpose),"locked_mana":c.and_then(|c|c.mana),"clauses":c.map(|c|&c.clauses),"active":c.and_then(|c|c.active),"current_hall_payable":c.is_some_and(|c|w.mana_clause_payable(c)),"convoke_masks":w.convoke_masks(),"improvise_potential":improvise::potential(&g,choice.item.source_card),"improvise_available":improvise::available(&w,goal.actor,choice.item.source_card),"resources":resources}));
        }
    }
    std::fs::write(output,serde_json::to_vec_pretty(&serde_json::json!({"config_index":14,"configuration":case,"prefix_decisions":ledger.len(),"ledger":ledger,"snapshot":g.snapshot(),"contexts":contexts})).unwrap()).unwrap();
    let _funding_budget = FundingBudget::new(128);
    let _teamwork_budget = Budget::new(256);
    let actions = g.legal_actions();
    assert!(!actions.is_empty());
}
