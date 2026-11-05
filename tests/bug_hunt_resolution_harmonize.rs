mod support;
use rustmage as _;
use rustmage_core::engine::*;
use support::*;
#[test]
fn bilbos_attack_permission_allows_mammoth_bellow_for_harmonize() {
    let p = Player::P0;
    let mut g = scenario(p);
    let bilbo = put(&mut g, p, "Bilbo, Thief in the Night", Zone::Battlefield);
    let helper = put(&mut g, p, "Surrak, Elusive Hunter", Zone::Battlefield);
    let bellow = put(&mut g, p, "Mammoth Bellow", Zone::Graveyard);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.players[p.index()].mana = [0, 1, 0, 1, 1, 0];

    act(&mut g, Action::Attack { card: bilbo });
    act(&mut g, Action::FinishAttackers);
    for _ in 0..12 {
        if matches!(g.state.pending, Pending::ResolutionCasts { .. }) {
            break;
        }
        choose(&mut g, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }

    assert!(matches!(g.state.pending, Pending::ResolutionCasts { .. }));
    assert_eq!(g.state.step, Step::Attackers);
    assert_eq!(g.state.players[p.index()].mana, [0, 1, 0, 1, 1, 0]);
    assert_eq!(g.object_view(helper).power, 4);
    assert!(!g.object_view(helper).tapped);
    let normal = Action::CastExiledDuringResolution {
        object: g.object_ref(bellow),
        face: CastFace::Normal,
    };
    let mut funded = g.clone();
    funded.state.players[p.index()].mana[5] = 1;
    assert!(funded.legal_actions().contains(&normal));

    reject(&mut g, normal);
    let harmonize = Action::AnnounceHarmonize { card: bellow };
    let mut no_helper = g.clone();
    no_helper.state.objects[helper as usize].tapped = true;
    reject(&mut no_helper, harmonize.clone());

    let index = g
        .legal_actions()
        .iter()
        .position(|a| *a == harmonize)
        .unwrap();
    let checkpoint = g.snapshot();
    let mut indexed = g.clone();
    indexed.apply_index(p, g.revision(), index).unwrap();
    act(&mut g, harmonize.clone());
    assert_eq!(indexed.snapshot(), g.snapshot());
    reject(&mut g, Action::ChooseHarmonizer { object: None });
    let object = g.object_ref(helper);
    act(
        &mut g,
        Action::ChooseHarmonizer {
            object: Some(object),
        },
    );
    for _ in 0..8 {
        match g.state.pending {
            Pending::CastCosts { .. } => {
                choose(&mut g, |a| matches!(a, Action::ChooseCastCost { .. }))
            }
            Pending::PayCosts { .. } => pay(&mut g),
            Pending::ResolutionCasts { .. } => break,
            ref pending => panic!("unexpected announcement decision {pending:?}"),
        }
    }
    assert!(g.object_view(helper).tapped);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(g.object_view(bellow).zone, Zone::Stack);
    assert_eq!(
        g.state.stack.last().unwrap().cast.unwrap().method,
        CastMethod::Harmonize
    );
    act(&mut g, Action::FinishResolutionCasts);
    settle(&mut g);
    assert_eq!(g.object_view(bellow).zone, Zone::Exile);
    assert!(
        g.observe(p)
            .objects
            .iter()
            .any(|o| o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && o.card.has_subtype("Elephant")
                && (o.power, o.toughness) == (5, 5))
    );
    indexed.restore(&checkpoint).unwrap();
    assert!(indexed.legal_actions().contains(&harmonize));
}
