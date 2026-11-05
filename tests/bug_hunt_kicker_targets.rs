mod support;
use rustmage as _;
use rustmage_core::engine::*;
use support::*;

#[test]
fn a_kicker_target_plan_requires_funding_on_the_same_cast_route() {
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Omniscience", Zone::Battlefield);
    let eagles = put(&mut g, p, "The Eagles Are Coming!", Zone::Hand);
    let normal = Action::AnnounceCast { card: eagles };
    let free = Action::AnnounceFree {
        card: eagles,
        face: CastFace::Normal,
        flash_surcharge: false,
    };
    let plot = Action::AnnouncePlotted { card: eagles };
    let mut plotted = g.clone();
    let reference = plotted.move_object(eagles, Zone::Exile).after;
    plotted.remember_plotted(p, reference);
    plotted.state.turn += 1;

    let mut regular = g.clone();
    regular.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
    let mut with_target = regular.clone();
    let target = put(&mut with_target, p, "Bear Cub", Zone::Battlefield);
    act(&mut with_target, normal.clone());
    reject(&mut with_target, Action::ChooseKicker { pay: true });
    act(&mut with_target, Action::ChooseKicker { pay: false });
    act(
        &mut with_target,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(&mut with_target);

    g.state.players[p.index()].mana = [0; 6];
    plotted.state.players[p.index()].mana = [0; 6];
    for (mut g, cast, mana) in [
        (g, free, [2, 0, 0, 0, 0, 2]),
        (plotted, plot, [2, 0, 0, 0, 0, 2]),
        (regular, normal, [3, 0, 0, 0, 0, 3]),
    ] {
        reject(&mut g, cast.clone());
        g.state.players[p.index()].mana = mana;
        act(&mut g, cast);
        reject(&mut g, Action::ChooseKicker { pay: false });
        act(&mut g, Action::ChooseKicker { pay: true });
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        assert_eq!(g.object_view(eagles).zone, Zone::Stack);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    }
}

#[test]
fn mode_menu_keeps_a_fundable_kicked_target_plan() {
    use rustmage_core::cards::authored::{CardExtras, CardRules};
    use rustmage_core::cards::{
        Card, Effect, Instruction, ManaCost, PlayerSet, SpellMode, TargetPlan, TargetRule,
        TargetSpec, fixture_rules,
    };

    const MODAL: CardRules = CardRules {
        effect: Effect::Modal(&[SpellMode {
            text: "Target a creature unless kicked",
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        }]),
        extras: &CardExtras {
            kicker_cost: Some(ManaCost {
                colored: [0; 6],
                generic: 2,
            }),
            kicked_targets: Some(TargetPlan::NONE),
            ..CardExtras::EMPTY
        },
        ..CardRules::EMPTY
    };
    let _rules = fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &MODAL)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Omniscience", Zone::Battlefield);
    let spell = put(&mut g, p, "Opt", Zone::Hand);
    let free = Action::AnnounceFree {
        card: spell,
        face: CastFace::Normal,
        flash_surcharge: false,
    };
    g.state.players[p.index()].mana = [0; 6];
    reject(&mut g, free.clone());
    g.state.players[p.index()].mana[5] = 2;
    act(&mut g, free);
    act(&mut g, Action::ChooseMode { mode: 0 });
    reject(&mut g, Action::ChooseKicker { pay: false });
    act(&mut g, Action::ChooseKicker { pay: true });
    pay(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Stack);
    settle(&mut g);
    assert_eq!(g.hand_count(p), 1);
}
