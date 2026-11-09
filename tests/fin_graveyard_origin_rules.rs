//! Shared battlefield permission to cast matching cards from one's graveyard.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const PAID_ORIGIN: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CastFromGraveyardWithLife {
        filter: CastFilter::Artifact,
        life: 3,
        entry_counter: CounterKind::Finality,
    }],
}];
const PROVIDER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: PAID_ORIGIN,
    extras: &CardExtras {
        static_ability_groups: PAID_ORIGIN,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn route(g: &Game, card: CardId) -> Action {
    Action::AnnounceGraveyardWithLife {
        object: g.object_ref(card),
        face: CastFace::Normal,
        method: CastMethod::Normal,
        flash_surcharge: false,
        life: 3,
        entry_counter: CounterKind::Finality,
    }
}

fn choose_cost(g: &mut Game, kind: CastCostClause) {
    let clause = g
        .observe(g.actor().unwrap())
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == kind && !row.paid)
        .unwrap() as u16;
    act(g, Action::ChooseCastCost { clause });
}

#[test]
fn paid_origin_checks_timing_ownership_and_joint_life_and_mana_funding() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PROVIDER)]);
    let p = Player::P1;
    let mut g = scenario(p);
    let provider = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    let artifact = put(&mut g, p, "Clamorous Ironclad", Zone::Graveyard);
    let other_artifact = put(&mut g, p.other(), "Clamorous Ironclad", Zone::Graveyard);
    let nonartifact = put(&mut g, p, "Hill Giant", Zone::Graveyard);
    let hidden = put(&mut g, p, "Clamorous Ironclad", Zone::Hand);
    let action = route(&g, artifact);
    reject(&mut g, action.clone());

    g.state.objects[provider as usize].controller = p;
    for id in [other_artifact, nonartifact, hidden] {
        let forbidden = route(&g, id);
        reject(&mut g, forbidden);
    }
    let view = g.observe(p.other());
    assert!(
        view.objects
            .iter()
            .any(|o| o.id == artifact && o.zone == Zone::Graveyard)
    );
    assert!(!view.objects.iter().any(|o| o.id == hidden));
    assert!(!view.actions.contains(&action));

    g.state.step = Step::BeginCombat;
    reject(&mut g, action.clone());
    g.state.step = Step::Main1;
    g.state.players[p.index()].life = 2;
    reject(&mut g, action.clone());
    g.state.players[p.index()].life = 3;
    assert!(g.legal_actions().contains(&action));
    g.state.players[p.index()].life = 4;
    g.state.players[p.index()].mana = [0; 6];
    reject(&mut g, action.clone());

    for _ in 0..3 {
        put(&mut g, p, "Forest", Zone::Battlefield);
    }
    put(&mut g, p, "Mountain", Zone::Battlefield);
    act(&mut g, action);
    for _ in 0..4 {
        choose(&mut g, |a| matches!(a, Action::ActivateMana { .. }));
    }
    act(&mut g, Action::BeginOrderedCosts);
    choose_cost(&mut g, CastCostClause::Life(3));
    assert_eq!(g.state.players[p.index()].life, 1);
    choose_cost(&mut g, CastCostClause::Mana);
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
    assert_eq!(g.counter_amounts(artifact).get(CounterKind::Finality), 1);
    g.move_object(artifact, Zone::Graveyard);
    assert_eq!(g.object_view(artifact).zone, Zone::Exile);
}

#[test]
fn paid_origin_receipt_survives_provider_departure_and_owned_branch_restore() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PROVIDER)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let provider = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let artifact = put(&mut g, p, "Interface Ace", Zone::Graveyard);
    let normal = Action::AnnounceCast { card: artifact };
    g.grant_graveyard_permission(p, g.object_ref(artifact), GraveyardCastKind::Cast);

    let action = route(&g, artifact);
    let origin = g.snapshot();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| a == &action).unwrap();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    act(&mut raw, action.clone());
    indexed.apply_index(p, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    for branch in [&mut raw, &mut indexed] {
        pay(branch);
        choose_cost(branch, CastCostClause::Life(3));
        branch.move_object(provider, Zone::Graveyard);
        settle(branch);
        assert_eq!(
            branch.counter_amounts(artifact).get(CounterKind::Finality),
            1
        );
        assert_eq!(branch.state.players[p.index()].life, 17);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&origin).unwrap();
    assert_eq!(
        indexed.apply_index(p, revision, index),
        Err(Error::StaleRevision)
    );

    act(&mut indexed, normal);
    pay(&mut indexed);
    settle(&mut indexed);
    assert_eq!(
        indexed.counter_amounts(artifact).get(CounterKind::Finality),
        0
    );
    assert_eq!(indexed.state.players[p.index()].life, 20);

    g.move_object(artifact, Zone::Exile);
    g.move_object(artifact, Zone::Graveyard);
    reject(&mut g, action);
    assert!(g.legal_actions().contains(&route(&g, artifact)));
}

#[test]
fn countered_paid_artifact_never_gives_a_later_cast_finality() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &PROVIDER)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let provider = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let artifact = put(&mut g, p, "Clamorous Ironclad", Zone::Graveyard);
    let negate = put(&mut g, p.other(), "Negate", Zone::Hand);
    let paid = route(&g, artifact);
    act(&mut g, paid);
    pay(&mut g);
    choose_cost(&mut g, CastCostClause::Life(3));
    act(&mut g, Action::Pass);
    cast(&mut g, negate, Some(Target::Spell(artifact)));
    settle(&mut g);
    assert_eq!(g.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(g.state.players[p.index()].life, 17);

    g.move_object(provider, Zone::Graveyard);
    g.grant_graveyard_permission(p, g.object_ref(artifact), GraveyardCastKind::Cast);
    cast(&mut g, artifact, None);
    settle(&mut g);
    assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
    assert_eq!(g.counter_amounts(artifact).get(CounterKind::Finality), 0);
    assert_eq!(g.state.players[p.index()].life, 17);
}
