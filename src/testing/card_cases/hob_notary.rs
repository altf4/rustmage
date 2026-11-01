//! Focused printed outcomes; shared engine invariants live in core tests.
//! Nonlegendary copiable values, nontoken recursion guard and current subtype mana.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass | Action::PutTrigger { .. } | Action::ChooseReplacement { .. }
            )
        });
    }
    panic!("Notary choice budget: {:?}", g.state.pending);
}
fn drive(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
    });
}
fn card() -> Card {
    Card::from_name("The Notary Hobbits").unwrap()
}
fn paid_entry(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "The Notary Hobbits", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 2, 3];
    cast(g, id, None);
    until(g, |g| g.on_battlefield(id));
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn tokens(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield && o.controller == p && o.is_token())
                .then_some(id as CardId)
        })
        .collect()
}

fn types(g: &mut Game, id: CardId, names: &'static [&'static str]) {
    let recipient = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types: names,
            additional: false,
            timestamp,
        });
}

#[test]
fn hob_notary_paid_entry_copies_exact_printed_values_not_counters_pump_tap_or_damage() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let id = paid_entry(&mut g, p);
        g.add_counter(id, CounterKind::PlusOne, 2);
        g.state.objects[id as usize].tapped = true;
        g.state.objects[id as usize].damage = 1;
        let spell = put(&mut g, p, "Giant Growth", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 0];
        cast(&mut g, spell, Some(Target::Card(id)));
        drive(&mut g);
        assert_eq!(stats(&g, id), (6, 6));
        assert!(g.object_view(id).is_legendary());
        let copies = tokens(&g, p);
        assert_eq!(copies.len(), 2);
        for copy in copies {
            let view = g.object_view(copy);
            assert_eq!(
                (view.power, view.toughness, view.colors, view.kinds),
                (1, 1, 16, 1)
            );
            assert!(!view.is_legendary() && !view.tapped && view.summoning_sick);
            assert!(view.has_subtype("Halfling") && view.has_subtype("Advisor"));
            assert_eq!(view.card.fixed_mana_value(), 5);
            assert_eq!(view.name(), "The Notary Hobbits");
            assert_eq!(view.owner, p);
            assert_eq!(view.counters.get(CounterKind::PlusOne), 0);
            assert_eq!(g.state.objects[copy as usize].damage, 0);
            assert_eq!(view.card.activations().len(), 1);
        }
        assert!(g.state.pending_triggers.is_empty());
        assert!(tokens(&g, p.other()).is_empty());
    }
}

#[test]
fn hob_notary_nontoken_copy_entry_preserves_other_copiable_exceptions() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let id = put(&mut g, p, "Bear Cub", Zone::Hand);
        let events = g.move_objects_raw(&[id], Zone::Battlefield);
        let mut recipe = CopyRecipe::new(card()).with_exceptions(CopyExceptions {
            artifact: true,
            base: Some((3, 4)),
            ..CopyExceptions::NONE
        });
        recipe.overrides = Some(CopyOverrides {
            name: Some("Notary blueprint"),
            legendary: Some(true),
            keywords: Keywords::REACH,
            ..CopyOverrides::NONE
        });
        g.apply_temporary_copy(g.object_ref(id), recipe, CopyDeadline::EndOfTurn);
        g.finish_zone_events(&events, &[]);
        g.give_priority();
        g.prepare_triggers();
        drive(&mut g);
        assert_eq!(tokens(&g, p).len(), 2);
        for copy in tokens(&g, p) {
            let v = g.object_view(copy);
            assert_eq!((v.power, v.toughness, v.kinds, v.colors), (3, 4, 17, 16));
            assert_eq!(v.name(), "Notary blueprint");
            assert!(v.has_subtype("Halfling") && v.has_subtype("Advisor"));
            assert!(!v.is_legendary());
            assert!(v.keywords.contains(Keywords::REACH));
            let copied = g.copiable_recipe(g.object_ref(copy));
            assert_eq!(copied.exceptions, recipe.exceptions);
            assert_eq!(copied.overrides.unwrap().legendary, Some(false));
            assert_eq!(
                copied.overrides.unwrap().name,
                recipe.overrides.unwrap().name
            );
        }
    }
}

#[test]
fn hob_notary_trigger_multiplication_and_token_replacement_multiply_without_recursion() {
    {
        let p = Player::P0;
        {
            let mut g = scenario(p);
            put(&mut g, p, "Bifur, Melodic Rider", Zone::Battlefield);
            put(&mut g, p, "Goldvein Pick", Zone::Battlefield);

            let id = put(&mut g, p, "The Notary Hobbits", Zone::Hand);
            let events = g.move_objects_raw(&[id], Zone::Battlefield);
            types(&mut g, id, &["Dwarf"]);
            g.finish_zone_events(&events, &[]);
            g.give_priority();
            g.prepare_triggers();
            drive(&mut g);
            assert!(g.state.players[p.index()].enduring_story);
            assert_eq!(tokens(&g, p).len(), { 4 });
            assert!(
                tokens(&g, p)
                    .into_iter()
                    .all(|id| g.has_subtype(id, "Halfling") && !g.has_subtype(id, "Dwarf")),
                "layer4 subtype overlay is not copiable"
            );
        }
    }
}
