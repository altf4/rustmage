mod support;
use rustmage as _;
use rustmage_core::engine::*;
use support::*;
#[test]
fn omniscience_can_waive_cruel_alliances_mana_cost_with_teamwork() {
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Omniscience", Zone::Battlefield);
    let helper = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let dragon = put(&mut g, p.other(), "Shivan Dragon", Zone::Battlefield);
    let alliance = put(&mut g, p, "Cruel Alliance", Zone::Hand);
    g.state.players[p.index()].mana = [0; 6];

    // The one available teammate is enough, but neither creature is eligible
    // without teamwork. There are no other ways to pay the spell's mana cost.
    assert!(g.object_view(helper).power >= 2);
    assert!(!g.object_view(helper).tapped);
    let free = Action::AnnounceFree {
        card: alliance,
        face: CastFace::Normal,
        flash_surcharge: false,
    };
    let paid = Action::AnnounceCast { card: alliance };
    let mut funded = g.clone();
    funded.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
    act(&mut funded, paid.clone());
    act(&mut funded, Action::ChooseTeamwork { pay: true });
    assert!(
        funded.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(dragon)
        }),
        "teamwork should make Shivan Dragon a legal target"
    );

    reject(&mut g, paid);
    let mut tapped_helper = g.clone();
    tapped_helper.state.objects[helper as usize].tapped = true;
    reject(&mut tapped_helper, free.clone());

    let index = g
        .legal_actions()
        .iter()
        .position(|action| *action == free)
        .unwrap();
    let checkpoint = g.snapshot();
    let mut indexed = g.clone();
    indexed.apply_index(p, g.revision(), index).unwrap();
    act(&mut g, free.clone());
    assert_eq!(indexed.snapshot(), g.snapshot());
    reject(&mut g, Action::ChooseTeamwork { pay: false });
    act(&mut g, Action::ChooseTeamwork { pay: true });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Card(dragon),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    for _ in 0..8 {
        match g.state.pending {
            Pending::CastCosts { .. } => {
                choose(&mut g, |a| matches!(a, Action::ChooseCastCost { .. }))
            }
            Pending::TeamworkPayment { .. } => {
                let object = g.object_ref(helper);
                act(&mut g, Action::ChooseTeammate { object });
                act(&mut g, Action::FinishTeamwork);
            }
            Pending::PayCosts { .. } => pay(&mut g),
            Pending::Priority => break,
            ref pending => panic!("unexpected announcement decision {pending:?}"),
        }
    }
    assert!(g.object_view(helper).tapped);
    assert_eq!(g.object_view(alliance).zone, Zone::Stack);
    settle(&mut g);
    assert_eq!(g.object_view(dragon).zone, Zone::Exile);
    assert_eq!(g.state.players[p.index()].life, 23);
    indexed.restore(&checkpoint).unwrap();
    assert!(indexed.legal_actions().contains(&free));
}
