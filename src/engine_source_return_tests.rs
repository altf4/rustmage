use super::tests::{act, cast, pass_pair, scenario, spawn};
use super::*;
const NAMES: [&str; 11] = [
    "Abzan Devotee",
    "Arcanis the Omnipotent",
    "Bestial Bloodline",
    "Blessed Ghoul",
    "Merchant of Many Hats",
    "Project Deathlok Soldier",
    "Reassembling Skeleton",
    "Summoned Dromedary",
    "Teacher's Pest",
    "Tunnel Rats",
    "Wingspan Stride",
];
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    let c = Card::from_name(n).unwrap();
    assert!(c.is_supported(), "{n}");
    let id = spawn(g, p, c, z);
    if z == Zone::Battlefield && c.aura().is_some() {
        let target = spawn(g, p, Card::GrizzlyBears, Zone::Battlefield);
        g.state.objects[id as usize].attached_to = Some(g.object_ref(target));
    }
    id
}
fn setup(p: Player) -> Game {
    let mut g = scenario();
    g.state.active = p;
    g.state.priority = p;
    for s in &mut g.state.players {
        s.mana = [20; 6];
    }
    g
}
fn slot(g: &Game, id: CardId) -> u16 {
    g.object_view(id)
        .card
        .activations()
        .iter()
        .position(|a| {
            a.program.iter().any(|p| {
                matches!(
                    p,
                    Instruction::ReturnSourceToHand(_)
                        | Instruction::ReturnSourceToBattlefield { .. }
                )
            })
        })
        .unwrap() as u16
}
fn replay(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let rev = g.revision();
    let p = g.actor().unwrap();
    let mut f = g.clone();
    act(&mut f, a.clone());
    act(g, a.clone());
    let e = g.snapshot();
    assert_eq!(f.snapshot(), e);
    g.restore(&s).unwrap();
    assert_eq!(g.apply_index(p, rev, 0), Err(Error::StaleRevision));
    act(g, a);
    assert_eq!(g.snapshot(), e);
}
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    let r = g.revision();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), s);
    assert_eq!(g.revision(), r);
}
fn action(g: &Game, id: CardId) -> Action {
    let index = slot(g, id);
    g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate{card,ability,target:None,..}if *card==id&&*ability==index)).expect("source return available")
}
#[test]
fn nine_graveyard_returns_pay_without_moving_source_then_resolve_exact_new_incarnation() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES
            .iter()
            .enumerate()
            .filter(|(i, _)| !matches!(i, 1 | 10))
        {
            let mut g = setup(p);
            let id = named(&mut g, p, name, Zone::Graveyard);
            let original = g.object_ref(id);
            let a = action(&g, id);
            let Action::Activate { payment, .. } = a else {
                unreachable!()
            };
            let before = g.state.players[p.index()].mana;
            replay(&mut g, a);
            assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            assert_eq!(g.object_ref(id), original);
            assert_eq!(
                g.state.players[p.index()].mana,
                std::array::from_fn(|n| before[n] - payment[n])
            );
            assert_eq!(g.state.stack[0].object, original);
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 0);
            pass_pair(&mut g);
            let dest = if matches!(i, 6 | 8 | 9) {
                Zone::Battlefield
            } else {
                Zone::Hand
            };
            let source = g.object_view(id);
            assert_eq!(source.zone, dest);
            assert_ne!(source.reference, original);
            assert_eq!(source.owner, p);
            assert_eq!(g.state.creatures_died_this_turn, 0);
            assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 0);
            assert!(g.state.stack.is_empty());
            if dest == Zone::Battlefield {
                assert!(source.tapped && source.summoning_sick);
                assert_eq!(source.counters, Counters::default());
                assert!(!g.observe(p.other()).known_hand.contains(&source.reference));
            } else {
                let public = g.observe(p.other());
                assert!(public.known_hand.contains(&source.reference));
                assert!(
                    public
                        .objects
                        .iter()
                        .any(|o| o.reference == source.reference && o.zone == Zone::Hand)
                );
            }
        }
    }
}
#[test]
fn source_return_zone_owner_and_sorcery_timing_are_authoritative() {
    for p in [Player::P0, Player::P1] {
        for (i, name) in NAMES.iter().enumerate() {
            let right = if matches!(i, 1 | 10) {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            };
            for z in [
                Zone::Hand,
                Zone::Battlefield,
                Zone::Graveyard,
                Zone::Exile,
                Zone::Library,
            ] {
                let mut g = setup(p);
                let id = named(&mut g, if z == right { p.other() } else { p }, name, z);
                let index = slot(&g, id);
                assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::Activate{card,ability,..}|Action::AnnounceActivate{card,ability,..}if *card==id&&*ability==index)));
                reject(
                    &mut g,
                    Action::AnnounceActivate {
                        card: id,
                        ability: index,
                        counter_payment: Counters::default(),
                    },
                );
            }
        }
    }
    for mode in 0..3 {
        for (i, name) in NAMES
            .iter()
            .enumerate()
            .filter(|(i, _)| !matches!(i, 1 | 10))
        {
            let mut g = setup(Player::P0);
            let id = named(&mut g, Player::P0, name, Zone::Graveyard);
            match mode {
                0 => g.state.step = Step::BeginCombat,
                1 => g.state.active = Player::P1,
                _ => {
                    let b = named(&mut g, Player::P0, "Grizzly Bears", Zone::Hand);
                    cast(&mut g, b, None);
                }
            }
            assert_eq!(g.legal_actions().iter().any(|a|matches!(a,Action::AnnounceActivate{card,ability,..}if *card==id&&*ability==slot(&g,id))),i!=7,"{name} {mode}");
        }
    }
}
#[test]
fn source_return_does_not_alias_reused_graveyard_or_battlefield_incarnations() {
    for (i, name) in NAMES.iter().enumerate() {
        let mut g = setup(Player::P0);
        let z = if matches!(i, 1 | 10) {
            Zone::Battlefield
        } else {
            Zone::Graveyard
        };
        let id = named(&mut g, Player::P0, name, z);
        let a = action(&g, id);
        replay(&mut g, a);
        let old = g.object_ref(id);
        g.move_object(id, Zone::Exile);
        g.move_object(id, z);
        if z == Zone::Battlefield && g.object_view(id).card.aura().is_some() {
            let target = g
                .state
                .objects
                .iter()
                .position(|o| o.zone == Zone::Battlefield && o.card == Card::GrizzlyBears)
                .unwrap() as CardId;
            g.state.objects[id as usize].attached_to = Some(g.object_ref(target));
        }
        let new = g.object_ref(id);
        assert_ne!(old, new);
        pass_pair(&mut g);
        assert_eq!(g.object_ref(id), new);
        assert_eq!(g.object_view(id).zone, z);
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn repeated_source_returns_are_independent_and_only_first_resolved_moves_source() {
    for name in [
        "Merchant of Many Hats",
        "Reassembling Skeleton",
        "Arcanis the Omnipotent",
    ] {
        let mut g = setup(Player::P0);
        let z = if name.starts_with("Arcanis") {
            Zone::Battlefield
        } else {
            Zone::Graveyard
        };
        let id = named(&mut g, Player::P0, name, z);
        let a = action(&g, id);
        replay(&mut g, a);
        let a = action(&g, id);
        replay(&mut g, a);
        assert_eq!(g.state.stack.len(), 2);
        assert_ne!(g.state.stack[0].id, g.state.stack[1].id);
        assert_eq!(g.state.stack[0].object, g.state.stack[1].object);
        pass_pair(&mut g);
        let new = g.object_ref(id);
        assert_eq!(
            g.object_view(id).zone,
            if name == "Reassembling Skeleton" {
                Zone::Battlefield
            } else {
                Zone::Hand
            }
        );
        pass_pair(&mut g);
        assert_eq!(g.object_ref(id), new);
        assert!(g.state.stack.is_empty());
        assert_eq!(
            g.state.known_hand.len(),
            usize::from(name != "Reassembling Skeleton")
        );
    }
}
#[test]
fn source_may_leave_during_owned_mana_payment_when_it_has_no_unpaid_source_cost() {
    for returns in [false, true] {
        let mut g = setup(Player::P0);
        let id = named(&mut g, Player::P0, "Reassembling Skeleton", Zone::Graveyard);
        let index = slot(&g, id);
        let old = g.object_ref(id);
        replay(
            &mut g,
            Action::AnnounceActivate {
                card: id,
                ability: index,
                counter_payment: Counters::default(),
            },
        );
        assert_eq!(g.observe(Player::P0).decision, "pay_costs");
        g.move_object(id, Zone::Exile);
        if returns {
            g.move_object(id, Zone::Graveyard);
        }
        let new = g.object_ref(id);
        assert_ne!(old, new);
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.state.stack[0].object, old);
        pass_pair(&mut g);
        assert_eq!(g.object_ref(id), new);
        assert_eq!(
            g.object_view(id).zone,
            if returns {
                Zone::Graveyard
            } else {
                Zone::Exile
            }
        );
    }
}
#[test]
fn source_return_capacity_failure_restores_resolution_without_refunding_paid_cost() {
    for name in [
        "Merchant of Many Hats",
        "Reassembling Skeleton",
        "Arcanis the Omnipotent",
    ] {
        let mut g = setup(Player::P0);
        let id = named(
            &mut g,
            Player::P0,
            name,
            if name.starts_with("Arcanis") {
                Zone::Battlefield
            } else {
                Zone::Graveyard
            },
        );
        g.state.objects[id as usize].generation = u64::MAX;
        let a = action(&g, id);
        replay(&mut g, a);
        let mana = g.state.players[0].mana;
        replay(&mut g, Action::Pass);
        let before = g.snapshot();
        let revision = g.revision();
        let p = g.actor().unwrap();
        for _ in 0..2 {
            assert!(matches!(
                g.apply(p, Action::Pass),
                Err(Error::CapacityExceeded(_))
            ));
            assert_eq!(g.snapshot(), before);
            assert_eq!(g.revision(), revision);
            assert_eq!(g.state.players[0].mana, mana);
        }
        let mut f = g.clone();
        assert!(matches!(
            f.apply(p, Action::Pass),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(f.snapshot(), before);
    }
}
#[test]
fn staged_hybrid_return_and_land_funding_keep_source_unmoved_until_resolution() {
    for p in [Player::P0, Player::P1] {
        for color in [0, 2] {
            let mut g = setup(p);
            let id = named(&mut g, p, "Blessed Ghoul", Zone::Graveyard);
            let original = g.object_ref(id);
            let index = slot(&g, id);
            replay(
                &mut g,
                Action::AnnounceActivate {
                    card: id,
                    ability: index,
                    counter_payment: Counters::default(),
                },
            );
            assert_eq!(g.observe(p).decision, "activation_cost");
            assert!(g.observe(p.other()).cost_choices.is_empty());
            let choice = g
                .observe(p)
                .cost_choices
                .iter()
                .position(|c| c.cost.colored[color] == 1)
                .unwrap();
            replay(
                &mut g,
                Action::ChooseActivationCost {
                    cost: choice as u16,
                },
            );
            let mut wrong = [0; 6];
            wrong[if color == 0 { 2 } else { 0 }] = 3;
            reject(&mut g, Action::PayCosts { payment: wrong });
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::PayCosts { .. }))
                .unwrap();
            replay(&mut g, a);
            assert_eq!(g.object_ref(id), original);
            pass_pair(&mut g);
            assert_eq!(g.object_view(id).zone, Zone::Hand);
        }
    }
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, "Tunnel Rats", Zone::Graveyard);
    for _ in 0..5 {
        named(&mut g, Player::P0, "Swamp", Zone::Battlefield);
    }
    g.state.players[0].mana = [0; 6];
    let index = slot(&g, id);
    replay(
        &mut g,
        Action::AnnounceActivate {
            card: id,
            ability: index,
            counter_payment: Counters::default(),
        },
    );
    while !g
        .legal_actions()
        .iter()
        .any(|a| matches!(a, Action::PayCosts { .. }))
    {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::TapMana { .. }))
            .unwrap();
        replay(&mut g, a);
        assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    }
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    replay(&mut g, a);
    assert_eq!(g.object_view(id).zone, Zone::Graveyard);
    pass_pair(&mut g);
    assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    assert!(g.object_view(id).tapped);
}
#[test]
fn arcanis_draw_ability_survives_actual_source_return_and_resolves_three_cards() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = named(&mut g, p, "Arcanis the Omnipotent", Zone::Battlefield);
        let draw = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a,Action::Activate{card,ability:0,..}if *card==id))
            .unwrap();
        replay(&mut g, draw);
        assert!(g.object_view(id).tapped);
        let a = action(&g, id);
        replay(&mut g, a);
        pass_pair(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Hand);
        assert_eq!(g.state.stack.len(), 1);
        pass_pair(&mut g);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 3);
        assert!(g.state.stack.is_empty());
    }
}
#[test]
fn actual_aura_bodies_keep_attachment_effects_and_known_return_recast_identity() {
    for p in [Player::P0, Player::P1] {
        for name in ["Bestial Bloodline", "Wingspan Stride"] {
            let mut g = setup(p);
            let target = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            let id = named(&mut g, p, name, Zone::Hand);
            cast(&mut g, id, Some(Target::Card(target)));
            pass_pair(&mut g);
            let bonus = if name == "Bestial Bloodline" { 2 } else { 1 };
            assert_eq!(g.object_view(target).power, 2 + bonus);
            assert_eq!(g.object_view(target).toughness, 2 + bonus);
            assert_eq!(
                g.keywords(target).contains(Keywords::FLYING),
                name == "Wingspan Stride"
            );
            assert_eq!(g.object_view(id).attached_to, Some(g.object_ref(target)));
            if name == "Bestial Bloodline" {
                let destroy = named(&mut g, p, "Withering Torment", Zone::Hand);
                cast(&mut g, destroy, Some(Target::Card(id)));
                pass_pair(&mut g);
                assert_eq!(g.object_view(id).zone, Zone::Graveyard);
            }
            let a = action(&g, id);
            replay(&mut g, a);
            pass_pair(&mut g);
            let hand = g.object_ref(id);
            assert_eq!(g.object_view(id).zone, Zone::Hand);
            assert_eq!(g.object_view(id).attached_to, None);
            assert!(g.observe(p.other()).known_hand.contains(&hand));
            assert_eq!(
                (g.object_view(target).power, g.object_view(target).toughness),
                (2, 2)
            );
            assert!(!g.keywords(target).contains(Keywords::FLYING));
            cast(&mut g, id, Some(Target::Card(target)));
            assert!(!g.state.known_hand.contains(&hand));
            pass_pair(&mut g);
            assert_ne!(g.object_ref(id), hand);
            assert_eq!(g.object_view(target).power, 2 + bonus);
        }
    }
}
#[test]
fn returned_creature_registers_entry_and_attack_observers_with_forced_tapped_state() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let observer = named(&mut g, p, "Hinterland Sanctifier", Zone::Battlefield);
        let id = named(&mut g, p, "Teacher's Pest", Zone::Graveyard);
        let a = action(&g, id);
        replay(&mut g, a);
        pass_pair(&mut g);
        assert!(g.object_view(id).tapped && g.object_view(id).summoning_sick);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, 0);
        assert!(g.keywords(id).contains(Keywords::MENACE));
        assert_eq!(g.state.stack[0].card, observer);
        pass_pair(&mut g);
        assert_eq!(g.state.players[p.index()].life, 21);
        // The returned creature's own printed attack observer must be registered.
        g.state.objects[id as usize].tapped = false;
        g.state.objects[id as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        replay(&mut g, Action::Attack { card: id });
        replay(&mut g, Action::FinishAttackers);
        pass_pair(&mut g);
        assert_eq!(g.state.players[p.index()].life, 22);
    }
}
#[test]
fn arcanis_actual_cast_respects_summoning_sickness_but_can_pay_untapped_independent_return() {
    let mut g = setup(Player::P0);
    let id = named(&mut g, Player::P0, "Arcanis the Omnipotent", Zone::Hand);
    cast(&mut g, id, None);
    pass_pair(&mut g);
    assert!(g.object_view(id).summoning_sick);
    assert_eq!(
        (g.object_view(id).power, g.object_view(id).toughness),
        (3, 4)
    );
    assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::Activate{card,ability:0,..}|Action::AnnounceActivate{card,ability:0,..}if *card==id)));
    let a = action(&g, id);
    replay(&mut g, a);
    pass_pair(&mut g);
    assert_eq!(g.object_view(id).zone, Zone::Hand);
}
#[test]
fn returning_tapped_does_not_change_normal_creature_cast_entry() {
    for name in ["Reassembling Skeleton", "Teacher's Pest", "Tunnel Rats"] {
        let mut g = setup(Player::P0);
        let id = named(&mut g, Player::P0, name, Zone::Graveyard);
        let a = action(&g, id);
        replay(&mut g, a);
        pass_pair(&mut g);
        assert!(g.object_view(id).tapped);
        let bounce = named(&mut g, Player::P0, "Unsummon", Zone::Hand);
        cast(&mut g, bounce, Some(Target::Card(id)));
        pass_pair(&mut g);
        cast(&mut g, id, None);
        pass_pair(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Battlefield);
        assert!(!g.object_view(id).tapped);
        assert!(g.object_view(id).summoning_sick);
    }
}
#[test]
fn abzan_conversion_stays_battlefield_only_and_once_per_turn_with_exact_three_colors() {
    for color in [0, 2, 4] {
        let mut g = setup(Player::P0);
        let id = named(&mut g, Player::P0, "Abzan Devotee", Zone::Battlefield);
        let mut production = [0; 6];
        production[color] = 1;
        let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::ActivateMana{card,ability:0,production:p,..}if *card==id&&*p==production)).unwrap();
        replay(&mut g, a);
        assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::ActivateMana{card,ability:0,..}|Action::AnnounceMana{card,ability:0,..}if *card==id)));
        g.move_object(id, Zone::Graveyard);
        assert!(!g.legal_actions().iter().any(
            |a| matches!(a,Action::ActivateMana{card,..}|Action::AnnounceMana{card,..}if *card==id)
        ));
        let a = action(&g, id);
        replay(&mut g, a);
        pass_pair(&mut g);
        assert_eq!(g.object_view(id).zone, Zone::Hand);
        assert!(g.observe(Player::P1).known_hand.contains(&g.object_ref(id)));
    }
}
