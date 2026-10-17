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
#[test]
fn aetherize_returns_only_current_attackers_ignoring_hexproof_and_ward() {
    for defender in [Player::P0, Player::P1] {
        let attacker = defender.other();
        let mut g = setup(attacker);
        g.state.players[defender.index()].mana = [30; 6];
        let bear = named(&mut g, attacker, "Grizzly Bears", Zone::Battlefield);
        let protected = named(&mut g, attacker, "Sheltered by Ghosts", Zone::Battlefield);
        // A noncreature Aura never joins the attacking group; after its exact
        // recipient leaves it goes to the graveyard through normal state actions.
        g.state.objects[protected as usize].attached_to = Some(g.object_ref(bear));
        let idle = named(&mut g, attacker, "Bear Cub", Zone::Battlefield);
        let ward = named(&mut g, attacker, "Hulking Raptor", Zone::Battlefield);
        let own = named(&mut g, defender, "Grizzly Bears", Zone::Battlefield);
        for id in [bear, idle, ward, own] {
            g.state.objects[id as usize].summoning_sick = false;
        }
        // Static hexproof is unrelated to membership in the non-targeted group.
        g.state.objects[bear as usize].gained_keywords = Keywords::HEXPROOF;
        let spell = named(&mut g, defender, "Aetherize", Zone::Hand);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        checked(&mut g, Action::Attack { card: bear });
        checked(&mut g, Action::Attack { card: ward });
        checked(&mut g, Action::FinishAttackers);
        checked(&mut g, Action::Pass);
        cast(&mut g, spell, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Hand);
        assert_eq!(g.object_view(ward).zone, Zone::Hand);
        assert_eq!(g.object_view(idle).zone, Zone::Battlefield);
        assert_eq!(g.object_view(own).zone, Zone::Battlefield);
        assert_eq!(g.object_view(protected).zone, Zone::Graveyard);
        assert!(g.state.attackers.is_empty());
    }
}
#[test]
fn rivers_rebuke_groups_current_controller_but_returns_to_exact_owner() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let stolen = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.objects[stolen as usize].controller = p.other();
        g.state.objects[stolen as usize].gained_keywords =
            Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE);
        let own = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let land = named(&mut g, p.other(), "Island", Zone::Battlefield);
        let artifact = named(&mut g, p.other(), "Mox Jasper", Zone::Battlefield);
        let foreign_aura = named(&mut g, p, "Ethereal Armor", Zone::Battlefield);
        g.state.objects[foreign_aura as usize].attached_to = Some(g.object_ref(stolen));
        let grouped_aura = named(&mut g, p.other(), "Ethereal Armor", Zone::Battlefield);
        g.state.objects[grouped_aura as usize].attached_to = Some(g.object_ref(stolen));
        let recipe = Card::catalogue()
            .find(|c| c.token_info().is_some_and(|t| t.subtypes == "Goblin"))
            .unwrap();
        let token = spawn(&mut g, p.other(), recipe, Zone::Battlefield);
        let spell = named(&mut g, p, "River's Rebuke", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Player(p.other())]);
        settle(&mut g);
        for id in [stolen, artifact, grouped_aura] {
            assert_eq!(g.object_view(id).zone, Zone::Hand);
        }
        assert_eq!(g.object_view(stolen).controller, p);
        assert_eq!(g.object_view(grouped_aura).controller, p.other());
        assert_eq!(g.object_view(own).zone, Zone::Battlefield);
        assert_eq!(g.object_view(land).zone, Zone::Battlefield);
        assert_eq!(g.object_view(foreign_aura).zone, Zone::Graveyard);
        assert_eq!(g.object_view(token).zone, Zone::Outside);
    }
}
#[test]
fn frozen_return_group_does_not_recapture_exiled_creature_returned_by_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let victim = named(&mut g, p, "Grizzly Bears", Zone::Battlefield);
        g.state.objects[victim as usize].controller = p.other();
        let light = named(&mut g, p, "Banishing Light", Zone::Hand);
        cast(&mut g, light, None, &[]);
        settle_on(&mut g, Some(Target::Card(victim)));
        assert_eq!(g.object_view(victim).zone, Zone::Exile);
        let spell = named(&mut g, p, "River's Rebuke", Zone::Hand);
        cast(&mut g, spell, None, &[Target::Player(p)]);
        settle(&mut g);
        assert_eq!(g.object_view(light).zone, Zone::Hand);
        assert_eq!(g.object_view(victim).zone, Zone::Battlefield);
        assert_eq!(g.object_view(victim).controller, p);
    }
}
#[test]
fn return_spell_cost_timing_and_empty_group_are_enforced() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let river = named(&mut g, p, "River's Rebuke", Zone::Hand);
        g.state.step = Step::CombatDamage;
        reject(&mut g, Action::AnnounceCast { card: river });
        g.state.step = Step::Main1;
        checked(&mut g, Action::AnnounceCast { card: river });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        for _ in 0..3 {
            if g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::PayCosts { .. }))
            {
                break;
            }
            checked(&mut g, Action::FinishTargets);
        }
        let mut payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        if let Action::PayCosts { payment, .. } = &mut payment {
            *payment = [0; 6];
        }
        reject(&mut g, payment);
        let valid = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, valid);
        settle(&mut g);
        assert_eq!(g.object_view(river).zone, Zone::Graveyard);
        let aether = named(&mut g, p, "Aetherize", Zone::Hand);
        cast(&mut g, aether, None, &[]);
        settle(&mut g);
        assert_eq!(g.object_view(aether).zone, Zone::Graveyard);
    }
}

#[test]
fn grouped_return_clears_death_grants_before_same_card_is_cast_again() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let grant = named(&mut g, p, "Fake Your Own Death", Zone::Hand);
        cast(&mut g, grant, None, &[Target::Card(bear)]);
        settle(&mut g);
        let original = g.object_ref(bear);
        assert_eq!(g.observe(p).temporary_abilities.len(), 1);
        let river = named(&mut g, p, "River's Rebuke", Zone::Hand);
        cast(&mut g, river, None, &[Target::Player(p)]);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Hand);
        assert!(g.observe(p).temporary_abilities.is_empty());
        cast(&mut g, bear, None, &[]);
        settle(&mut g);
        assert_ne!(g.object_ref(bear), original);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        let stab = named(&mut g, p, "Stab", Zone::Hand);
        cast(&mut g, stab, None, &[Target::Card(bear)]);
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert!(g.state.objects.iter().all(|o| {
            o.zone != Zone::Battlefield
                || !o
                    .card
                    .token_info()
                    .is_some_and(|t| t.name == "Treasure Token")
        }));
    }
}
