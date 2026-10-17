//! Shared qualified targeting and live threshold regressions using gated production definitions.
use super::ability_timeline_tests::{cast, checked, named, settle, setup};
use super::tests::spawn;
use super::*;
fn fixture(g: &mut Game, p: Player, name: &str) -> CardId {
    let card = Card::from_name(name).unwrap();
    spawn(g, p, card, Zone::Battlefield)
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn remove_abilities(g: &mut Game, p: Player, id: CardId) -> ObjectRef {
    g.state.priority = p;
    let aura = named(g, p, "Stop Cold", Zone::Hand);
    cast(g, aura, None, &[Target::Card(id)]);
    settle(g);
    g.object_ref(aura)
}
#[test]
fn qualified_hexproof_color_kind_matrix_and_atomic_paid_paths_both_seats() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let grace = fixture(&mut g, p, "Knight of Grace");
        let malice = fixture(&mut g, p, "Knight of Malice");
        let elenda = fixture(&mut g, p, "Elenda, Saint of Dusk");
        let murder = named(&mut g, p.other(), "Murder", Zone::Hand);
        let white = named(&mut g, p.other(), "Pacifism", Zone::Hand);
        let red = named(&mut g, p.other(), "Burst Lightning", Zone::Hand);
        let sorcery = named(&mut g, p.other(), "Feed the Swarm", Zone::Hand);
        for (source, target, legal) in [
            (murder, grace, false),
            (murder, malice, true),
            (murder, elenda, false),
            (white, grace, true),
            (white, malice, false),
            (white, elenda, true),
            (red, grace, true),
            (red, malice, true),
            (red, elenda, false),
            (sorcery, grace, false),
            (sorcery, malice, true),
            (sorcery, elenda, true),
        ] {
            assert_eq!(
                g.target_matches(
                    TargetRule::Creature,
                    p.other(),
                    g.object_ref(source),
                    Target::Card(target)
                ),
                legal,
                "source {:?}, target {:?}",
                g.state.objects[source as usize].card,
                g.state.objects[target as usize].card
            );
            assert!(g.target_matches(
                TargetRule::Creature,
                p,
                g.object_ref(source),
                Target::Card(target)
            ));
        }
        // An available alternate target keeps announcement open, exposing the same filter.
        named(&mut g, p, "Bear Cub", Zone::Battlefield);
        g.state.active = p.other();
        g.state.priority = p.other();
        reject(
            &mut g,
            Action::Cast {
                card: murder,
                target: Some(Target::Card(grace)),
                payment: [0, 0, 2, 0, 0, 1],
            },
        );
        checked(&mut g, Action::AnnounceCast { card: murder });
        assert!(!g.legal_actions().contains(&Action::ChooseTarget {
            target: Target::Card(grace)
        }));
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(grace),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(malice),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert_eq!(g.object_view(malice).zone, Zone::Graveyard);
        assert_eq!(g.object_view(grace).zone, Zone::Battlefield);
        // Opponent's hand identity remains hidden throughout owned replay checks.
        let hidden = named(&mut g, p, "Shore Up", Zone::Hand);
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == hidden));
    }
}
#[test]
fn qualified_hexproof_removed_and_restored_before_actual_resolution() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let grace = fixture(&mut g, p, "Knight of Grace");
        let aura = remove_abilities(&mut g, p.other(), grace);
        assert!(!g.keywords(grace).contains(Keywords::FIRST_STRIKE));
        let murder = named(&mut g, p.other(), "Murder", Zone::Hand);
        assert!(g.target_matches(
            TargetRule::Creature,
            p.other(),
            g.object_ref(murder),
            Target::Card(grace)
        ));
        g.state.active = p.other();
        g.state.priority = p.other();
        cast(&mut g, murder, None, &[Target::Card(grace)]);
        let snapshot = g.snapshot();
        g.move_object(aura.card, Zone::Graveyard);
        assert!(!g.target_matches(
            TargetRule::Creature,
            p.other(),
            g.object_ref(murder),
            Target::Card(grace)
        ));
        settle(&mut g);
        assert_eq!(g.object_view(grace).zone, Zone::Battlefield);
        let result = g.snapshot();
        g.restore(&snapshot).unwrap();
        g.move_object(aura.card, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.snapshot(), result);
    }
}
#[test]
fn qualified_hexproof_source_current_characteristics_and_exact_departure_lki() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let grace = fixture(&mut g, p, "Knight of Grace");
        let elenda = fixture(&mut g, p, "Elenda, Saint of Dusk");
        let fire = named(&mut g, p.other(), "Fanatical Firebrand", Zone::Battlefield);
        let mut item = g.activation_item(p.other(), fire, 0);
        // A live source is queried, never the packet's immutable ability definition.
        g.state.objects[fire as usize].card = Card::from_name("Vampire Nighthawk").unwrap();
        assert!(!g.item_target_matches(&item, TargetRule::Creature, Target::Card(grace)));
        assert!(g.item_target_matches(&item, TargetRule::Creature, Target::Card(elenda)));
        // Frozen departed source survives a new, differently colored incarnation.
        item.source_facts = g.damage_facts(fire);
        let old = item.object;
        g.move_object(fire, Zone::Graveyard);
        g.move_object(fire, Zone::Battlefield);
        g.state.objects[fire as usize].card = Card::from_name("Fanatical Firebrand").unwrap();
        assert_ne!(old, g.object_ref(fire));
        assert!(!g.item_target_matches(&item, TargetRule::Creature, Target::Card(grace)));
        // Instant qualification uses the source's type, including a departed instant.
        let instant = named(&mut g, p.other(), "Murder", Zone::Hand);
        let mut instant_item = g.activation_item(p.other(), instant, 0);
        instant_item.source_facts = g.damage_facts(instant);
        g.move_object(instant, Zone::Graveyard);
        assert!(!g.item_target_matches(&instant_item, TargetRule::Creature, Target::Card(elenda)));
        // Abilities of ordinary permanents are not instant spells merely because they use the stack.
        assert!(g.item_target_matches(&item, TargetRule::Creature, Target::Card(elenda)));
    }
}
#[test]
fn qualified_hexproof_live_color_presence_and_starting_life_thresholds() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let grace = fixture(&mut g, p, "Knight of Grace");
        assert_eq!(g.stats(grace).power, 2);
        let black = named(&mut g, p.other(), "Vampire Nighthawk", Zone::Hand);
        assert_eq!(g.stats(grace).power, 2);
        g.move_object(black, Zone::Battlefield);
        assert_eq!(g.stats(grace).power, 3);
        let other = named(&mut g, p, "Vampire Nighthawk", Zone::Battlefield);
        assert_eq!(g.stats(grace).power, 3); // Boolean, never per-permanent scaling.
        g.move_object(black, Zone::Graveyard);
        g.move_object(other, Zone::Graveyard);
        assert_eq!(g.stats(grace).power, 2);
        let elenda = fixture(&mut g, p, "Elenda, Saint of Dusk");
        for (life, stat, menace) in [
            (STARTING_LIFE, 4, false),
            (STARTING_LIFE + 1, 5, true),
            (STARTING_LIFE + 9, 5, true),
            (STARTING_LIFE + 10, 10, true),
            (STARTING_LIFE - 1, 4, false),
        ] {
            g.state.players[p.index()].life = life;
            assert_eq!(
                g.stats(elenda),
                Stats {
                    power: stat,
                    toughness: stat
                }
            );
            assert_eq!(g.keywords(elenda).contains(Keywords::MENACE), menace);
            assert!(g.keywords(elenda).contains(Keywords::LIFELINK));
        }
        g.state.players[p.other().index()].life = STARTING_LIFE + 10;
        assert_eq!(g.stats(elenda).power, 4);
        g.state.objects[elenda as usize].controller = p.other();
        assert_eq!(g.stats(elenda).power, 10);
        let curse = remove_abilities(&mut g, p, elenda);
        assert_eq!(g.stats(elenda).power, 4);
        assert!(!g.keywords(elenda).contains(Keywords::MENACE));
        g.move_object(curse.card, Zone::Graveyard);
        assert_eq!(g.stats(elenda).power, 10);
        assert_eq!(std::mem::size_of::<Object>(), 144);
    }
}

#[test]
fn qualified_hexproof_alternate_face_preflight_and_paid_source_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elenda = fixture(&mut g, p, "Elenda, Saint of Dusk");
        let physical = named(
            &mut g,
            p.other(),
            "Obyra's Attendants // Desperate Parry",
            Zone::Hand,
        );
        let definition = g.state.objects[physical as usize]
            .card
            .adventure_spell()
            .unwrap();
        g.state.priority = p.other();
        assert!(
            !g.legal_actions()
                .contains(&Action::AnnounceAdventure { card: physical })
        );
        let bear = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceAdventure { card: physical })
        );
        checked(&mut g, Action::AnnounceAdventure { card: physical });
        assert_eq!(
            g.state.announcing.as_ref().unwrap().item.source_card,
            definition
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(elenda),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            checked(&mut g, Action::FinishTargets);
        }
        let pay = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, pay);
        settle(&mut g);
        assert_eq!(g.object_view(physical).zone, Zone::Exile);
        let fire = named(&mut g, p.other(), "Fanatical Firebrand", Zone::Battlefield);
        g.state.priority = p.other();
        let activate = g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate {card,target:Some(Target::Card(id)),..} if *card==fire && *id==elenda)).unwrap();
        checked(&mut g, activate);
        assert_eq!(g.object_view(fire).zone, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.object_view(elenda).damage, 1);
    }
}
#[test]
fn qualified_hexproof_announcement_uses_original_source_not_moved_target_anchor() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let grace = fixture(&mut g, p, "Knight of Grace");
        let source = named(&mut g, p.other(), "Vampire Nighthawk", Zone::Battlefield);
        let mut item = g.activation_item(p.other(), source, 0);
        let old = item.object;
        let event = g.move_object(source, Zone::Graveyard);
        item.source_move = Some(Box::new(TriggerSourceMoveView {
            before: old,
            after: event.after,
            origin: Zone::Battlefield,
            destination: Zone::Graveyard,
        }));
        g.state.objects[source as usize].card = Card::from_name("Fanatical Firebrand").unwrap();
        let plan = TargetPlan::Single(TargetSpec::one(TargetRule::Creature));
        assert!(!g.item_targets_available(&item, plan));
        assert!(
            !g.visit_item_targets(&item, TargetRule::Creature, |target| target
                == Target::Card(grace))
        );
        assert!(!g.item_target_matches(&item, TargetRule::Creature, Target::Card(grace)));
        // Ability controller remains captured even when a still-live source changes sides.
        let live = named(&mut g, p.other(), "Vampire Nighthawk", Zone::Battlefield);
        let item = g.activation_item(p.other(), live, 0);
        g.state.objects[live as usize].controller = p;
        assert!(!g.item_target_matches(&item, TargetRule::Creature, Target::Card(grace)));
    }
}
