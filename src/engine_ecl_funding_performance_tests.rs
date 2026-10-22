//! A real vivid-deck negative funding search and a legal restricted-mana converter.
use super::super::ability_timeline_tests::{checked, named, settle, setup};
use super::*;

fn position(p: Player, converter: bool) -> (Game, CardId, CardId) {
    let mut g = setup(p);
    for name in [
        "Forest",
        "Swamp",
        "Plains",
        "Great Forest Druid",
        "Great Forest Druid",
        "Flamebraider",
    ] {
        let id = named(&mut g, p, name, Zone::Battlefield);
        g.state.objects[id as usize].summoning_sick = false;
    }
    let realms = named(&mut g, p, "Eclipsed Realms", Zone::Battlefield);
    g.set_entry_value(
        g.object_ref(realms),
        EntryChoiceValue::CreatureType(crate::cards::CreatureType::from_name("Elf").unwrap()),
    );
    let druid = if converter {
        named(&mut g, p, "Foraging Wickermaw", Zone::Battlefield)
    } else {
        named(&mut g, p, "Bear Cub", Zone::Battlefield)
    };
    // An unrelated real attachment must not disable the negative shortcut.
    // On the converter it makes the activated source an Elemental, so the
    // restricted lot can pay that activation and become unrestricted mana.
    let dagger = named(&mut g, p, "Stalactite Dagger", Zone::Battlefield);
    assert!(g.attach_equipment(g.object_ref(dagger), g.object_ref(druid)));
    let aurora = named(&mut g, p, "Aurora Awakener", Zone::Hand);
    let prism = named(&mut g, p, "Prismabasher", Zone::Hand);
    g.state.players[p.index()].mana = [0; 6];
    (g, aurora, prism)
}
fn world_for(g: &Game, p: Player, card: CardId) -> (FundingState<'_>, FundingGoal) {
    let goal = FundingGoal {
        actor: p,
        required: [0, 0, 0, 0, 1, 0],
        generic: 6,
        reserve: None,
        source: None,
    };
    let mut world = FundingState::from_game(g, &goal);
    world.purpose = mana_lots::Purpose::Spell(mana_lots::Source {
        card: g.state.objects[card as usize].card,
        foreign_stash: false,
        exile_any_mana: false,
        mana_value: 7,
        kinds: 1,
        subtypes: GrantedSubtypes::default(),
    });
    (world, goal)
}
#[test]
fn restricted_quantity_bound_rejects_impossible_aurora_without_search_and_keeps_elemental() {
    for p in [Player::P0, Player::P1] {
        let (mut g, aurora, prism) = position(p, false);
        let (world, goal) = world_for(&g, p, aurora);
        assert!(world.total_mana_upper() >= 7);
        assert_eq!(world.spendable_mana_upper(), Some(6));
        assert!(!world.optimistic(&goal));
        let flame = g
            .state
            .objects
            .iter()
            .position(|object| {
                object.zone == Zone::Battlefield
                    && object.card == Card::from_name("Flamebraider").unwrap()
            })
            .unwrap() as CardId;
        checked(
            &mut g,
            Action::ActivateMana {
                card: flame,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production: [0, 0, 0, 2, 0, 0],
            },
        );
        let (world, goal) = world_for(&g, p, aurora);
        assert_eq!(
            world.spendable_mana_upper(),
            Some(6),
            "existing restricted lots remain ineligible"
        );
        assert!(!world.optimistic(&goal));
        let before = g.snapshot();
        let actions = g.legal_actions();
        assert!(!actions.contains(&Action::AnnounceCast { card: aurora }));
        assert!(actions.contains(&Action::AnnounceCast { card: prism }));
        assert_eq!(g.snapshot(), before);
        let revision = g.revision();
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        assert_eq!(
            branch.apply(p, Action::AnnounceCast { card: aurora }),
            Err(Error::IllegalAction)
        );
        assert_eq!(branch.snapshot(), before);
        assert_eq!(branch.revision(), revision);
        assert_eq!(
            g.apply(p, Action::AnnounceCast { card: aurora }),
            Err(Error::IllegalAction)
        );
        assert_eq!(g.snapshot(), before);
    }
}
#[test]
fn restricted_mana_converter_fallback_preserves_paid_aurora_and_owned_actions() {
    for p in [Player::P0, Player::P1] {
        let (mut g, aurora, _) = position(p, true);
        // Freeze the six unrestricted contributions in the pool; the positive
        // regression exercises conversion rather than unrelated color ordering.
        for object in &mut g.state.objects {
            if object.zone == Zone::Battlefield
                && object.controller == p
                && object.card != Card::from_name("Flamebraider").unwrap()
            {
                object.tapped = true;
            }
        }
        g.state.players[p.index()].mana = [0, 0, 0, 0, 6, 0];
        let (world, goal) = world_for(&g, p, aurora);
        assert_eq!(world.spendable_mana_upper(), None);
        assert!(world.reachable(&goal, &mut HashSet::new()));
        checked(&mut g, Action::AnnounceCast { card: aurora });
        for _ in 0..20 {
            if g.state.announcing.is_none() {
                break;
            }
            let actions = g.legal_actions();
            let action = actions
                .iter()
                .find(|a| matches!(a, Action::FinishTargets | Action::PayCosts { .. }))
                .or_else(|| {
                    actions
                        .iter()
                        .find(|a| matches!(a, Action::TapMana { .. } | Action::ActivateMana { .. }))
                })
                .expect("legal funding plan has a next action")
                .clone();
            checked(&mut g, action);
        }
        assert!(g.state.announcing.is_none());
        settle(&mut g);
        assert_eq!(g.state.objects[aurora as usize].zone, Zone::Battlefield);
    }
}
