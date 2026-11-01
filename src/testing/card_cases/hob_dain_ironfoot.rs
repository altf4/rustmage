//! Focused printed outcomes; shared engine invariants live in core tests.
//! Reflexive Axe creation and equipped-attacker grants.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..120 {
        if ready(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ChooseEquipmentAttachment { equipment: Some(_) }
            )
        });
    }
    panic!("Dain Ironfoot decision budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.ordering_triggers.is_empty()
            && g.state.resolving.is_none()
    });
}
fn axes(g: &Game, p: Player) -> Vec<CardId> {
    let card = Card::from_name("@token/hob-axe").unwrap();
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield && o.controller == p && o.card == card)
                .then_some(i as CardId)
        })
        .collect()
}
fn start(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Dáin Ironfoot", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
    cast(g, id, None);
    until(g, |g| matches!(g.state.pending, Pending::Targets { .. }));
    id
}
#[test]
fn dain_ironfoot_creation_then_reflexive_target_and_exact_attachment() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let friendly = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let id = start(&mut g, p);
        assert_eq!(stats(&g, id), (1, 4));
        let equipment = axes(&g, p);
        assert_eq!(equipment.len(), 1);
        let axe = equipment[0];
        for viewer in [p, p.other()] {
            let announced = g.observe(viewer).announcing.unwrap();
            let json = serde_json::to_value(announced.reflexive_subject.unwrap()).unwrap();
            assert!(json.get("additional_objects").is_none());
        }
        assert_eq!(g.object_view(axe).card.colors(), 0);
        assert!(g.has_subtype(axe, "Equipment"));
        assert_eq!(g.object_view(axe).attached_to, None);
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(p),
            },
        );

        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(friendly),
            },
        );
        finish(&mut g);
        assert_eq!(g.object_view(axe).attached_to, Some(g.object_ref(friendly)));
        assert_eq!(stats(&g, friendly), (3, 2));
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        act(
            &mut g,
            Action::AnnounceActivate {
                card: axe,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
        pay(&mut g);
        finish(&mut g);
        assert_eq!(g.object_view(axe).attached_to, Some(g.object_ref(id)));
        assert_eq!(stats(&g, id), (2, 4));
    }
}

#[test]
fn dain_ironfoot_attacking_equipped_set_is_evaluated_on_resolution() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let equipped = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let initially_unequipped = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let nonattacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let id = start(&mut g, p);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(equipped),
            },
        );
        finish(&mut g);
        let axe = axes(&g, p)[0];
        g.state.objects[id as usize].summoning_sick = false;
        g.state.active = p;
        g.state.priority = p;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        for card in [id, equipped, initially_unequipped] {
            act(&mut g, Action::Attack { card });
        }
        act(&mut g, Action::FinishAttackers);
        until(&mut g, |g| {
            g.state
                .stack
                .iter()
                .any(|item| matches!(item.kind, StackKind::TriggeredAbility { ability: 1 }))
        });
        assert!(g.attach_equipment(g.object_ref(axe), g.object_ref(initially_unequipped)));
        g.move_object(id, Zone::Graveyard);
        finish(&mut g);
        assert!(
            !g.object_view(equipped)
                .keywords
                .contains(Keywords::DOUBLE_STRIKE)
        );
        assert!(
            g.object_view(initially_unequipped)
                .keywords
                .contains(Keywords::DOUBLE_STRIKE)
        );
        assert!(
            !g.object_view(nonattacker)
                .keywords
                .contains(Keywords::DOUBLE_STRIKE)
        );
        g.finish_cleanup();
        assert!(
            !g.object_view(initially_unequipped)
                .keywords
                .contains(Keywords::DOUBLE_STRIKE)
        );
    }
}
