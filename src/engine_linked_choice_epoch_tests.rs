//! Linked ability instances survive in old packets without binding newly gained text.
use super::ability_timeline_tests::{checked, named, settle, setup};
use super::*;
fn packet(g: &mut Game, source: CardId) -> SpellView {
    // Isolate the ordinary departure-trigger constructor at the typed boundary.
    assert!(g.queue_trigger(&g.object_view(source), 1));
    g.state.pending_triggers.pop().unwrap()
}
fn resolve(g: &mut Game, item: SpellView) {
    g.state.stack.push(item);
    g.state.pending = Pending::Priority;
    g.state.priority = g.state.active;
    g.state.passes = 0;
    checked(g, Action::Pass);
    checked(g, Action::Pass);
    settle(g);
}
#[test]
fn linked_choice_epochs_old_and_new_packets_return_only_their_own_exact_links() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let source = named(&mut g, p, "Champion of the Path", Zone::Battlefield);
        let object = g.object_ref(source);
        let old_card = named(&mut g, p, "Bear Cub", Zone::Exile);
        g.remember_linked_exile_for(object, &[old_card]);
        let old_packet = packet(&mut g, source);
        assert_eq!(old_packet.ability_origin(), 0);
        // Typed boundary for gaining a new copy-effect ability without a new object.
        let definition = g.state.objects[source as usize].card;
        g.state.copy_recipes = Some(
            vec![copies::CopyRow {
                native: None,
                loyalty_used_turn: None,
                object,
                recipe: CopyRecipe::new(definition),
                ability_origin: 42,
            }]
            .into(),
        );
        let new_card = named(&mut g, p.other(), "Island", Zone::Exile);
        g.remember_linked_exile_for(object, &[new_card]);
        let new_packet = packet(&mut g, source);
        assert_eq!(new_packet.ability_origin(), 42);
        assert_eq!(g.object_view(source).copy_ability_origin, 42);
        assert_eq!(
            g.observe(p)
                .linked_exile
                .iter()
                .map(|row| row.ability_origin)
                .collect::<Vec<_>>(),
            vec![0, 42]
        );
        let saved = g.snapshot();
        let mut branch = {
            let parent = g.clone();
            parent.clone()
        };
        resolve(&mut branch, new_packet.clone());
        resolve(&mut g, new_packet.clone());
        assert_eq!(g.snapshot(), branch.snapshot());
        assert_eq!(g.state.objects[new_card as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[new_card as usize].owner, p.other());
        assert_eq!(g.state.objects[old_card as usize].zone, Zone::Exile);
        g.restore(&saved).unwrap();
        resolve(&mut g, old_packet);
        assert_eq!(g.state.objects[old_card as usize].zone, Zone::Hand);
        assert_eq!(g.state.objects[new_card as usize].zone, Zone::Exile);
    }
}
#[test]
fn linked_choice_epoch_packet_keeps_recipe_and_origin_after_source_origin_changes() {
    let mut g = setup(Player::P0);
    let source = named(
        &mut g,
        Player::P0,
        "Champion of the Path",
        Zone::Battlefield,
    );
    let object = g.object_ref(source);
    let definition = g.state.objects[source as usize].card;
    g.state.copy_recipes = Some(
        vec![copies::CopyRow {
            native: None,
            loyalty_used_turn: None,
            object,
            recipe: CopyRecipe::new(definition),
            ability_origin: 42,
        }]
        .into(),
    );
    let exiled = named(&mut g, Player::P0, "Forest", Zone::Exile);
    g.remember_linked_exile_for(object, &[exiled]);
    let captured = packet(&mut g, source);
    let before = g.snapshot();
    g.state.copy_recipes.as_mut().unwrap()[0].ability_origin = 84;
    assert_eq!(g.copy_ability_origin(object), 84);
    assert_eq!(captured.ability_origin(), 42);
    assert_eq!(
        captured
            .source_copy_recipe
            .as_ref()
            .unwrap()
            .recipe
            .definition,
        definition
    );
    resolve(&mut g, captured.clone());
    assert_eq!(g.state.objects[exiled as usize].zone, Zone::Hand);
    g.restore(&before).unwrap();
    g.state.copy_recipes.as_mut().unwrap()[0].ability_origin = 84;
    g.remember_linked_exile_for_origin(object, captured.ability_origin(), &[exiled]);
    assert!(
        g.observe(Player::P0)
            .linked_exile
            .iter()
            .all(|row| row.ability_origin == 42)
    );
}
