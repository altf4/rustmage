//! Exact Saga-granted landfall and frozen Elf recipients.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..100 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("valley decision budget {:?}", g.state.pending);
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
fn step(g: &mut Game) {
    g.begin_step(Step::Main1);
    g.give_priority();
    g.prepare_triggers();
}
fn tokens(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.card == Card::from_name("@token/a79da9ee6d2f00d35c342024").unwrap())
            .then_some(i as CardId)
        })
        .collect()
}
#[test]
fn down_valley_paid_search_granted_landfall_and_chapters() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let land = put(&mut g, p, "Forest", Zone::Library);
        let bad = put(&mut g, p, "Bear Cub", Zone::Library);
        let saga = put(&mut g, p, "Down in the Valley", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 2];
        cast(&mut g, saga, None);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        assert!(g.observe(p).search_choice.is_some());
        assert!(g.observe(p.other()).search_choice.is_none());
        reject(&mut g, Action::ChooseSearch { card: bad });
        contracts::branch_contract(g.clone());
        act(&mut g, Action::ChooseSearch { card: land });
        act(&mut g, Action::FinishSearch);
        finish(&mut g);
        assert_eq!(g.object_view(land).zone, Zone::Hand);
        assert!(
            g.observe(p.other())
                .known_hand
                .contains(&g.object_ref(land))
        );
        act(&mut g, Action::PlayLand { card: land });
        finish(&mut g);
        assert!(tokens(&g, p).is_empty());
        step(&mut g);
        finish(&mut g);
        let enemy = put(&mut g, p.other(), "Forest", Zone::Hand);
        g.move_object(enemy, Zone::Battlefield);
        g.give_priority();
        finish(&mut g);
        assert!(tokens(&g, p).is_empty());
        let land2 = put(&mut g, p, "Island", Zone::Hand);
        g.move_object(land2, Zone::Battlefield);
        g.give_priority();
        finish(&mut g);
        let elves = tokens(&g, p);
        assert_eq!(elves.len(), 1);
        assert_eq!(stats(&g, elves[0]), (1, 1));
        assert_eq!(g.object_view(elves[0]).card.colors(), 16);
        assert!(g.has_subtype(elves[0], "Elf"));
        let kindred = put(&mut g, p, "Morcant's Eyes", Zone::Battlefield);
        let recipient = g.object_ref(kindred);
        assert!(g.has_subtype(kindred, "Elf"));
        step(&mut g);
        finish(&mut g);
        assert!(
            g.object_view(kindred)
                .keywords
                .contains(Keywords::VIGILANCE)
        );
        g.apply_temporary_copy(
            recipient,
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        assert_eq!(stats(&g, kindred), (3, 2));
        assert_eq!(stats(&g, elves[0]), (2, 1));
        assert!(
            g.object_view(elves[0])
                .keywords
                .contains(Keywords::VIGILANCE)
        );
        let late = put(&mut g, p, "Boughside Wanderers", Zone::Battlefield);
        assert!(!g.object_view(late).keywords.contains(Keywords::VIGILANCE));
        step(&mut g);
        finish(&mut g);
        assert_eq!(stats(&g, elves[0]), (3, 1));
        assert!(g.object_view(late).keywords.contains(Keywords::VIGILANCE));
        assert_eq!(g.object_view(saga).zone, Zone::Graveyard);
        g.finish_cleanup();
        assert_eq!(stats(&g, elves[0]), (1, 1));
        assert!(
            !g.object_view(elves[0])
                .keywords
                .contains(Keywords::VIGILANCE)
        );
    }
}

#[test]
fn down_valley_grant_suppression_recovery_blink_and_simultaneous_lands() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let saga = put(&mut g, p, "Down in the Valley", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 2];
        cast(&mut g, saga, None);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        act(&mut g, Action::FinishSearch);
        finish(&mut g);
        step(&mut g);
        finish(&mut g);
        let source = g.object_ref(saga);
        g.blank_and_grant_haste(source, source);
        let blocked = put(&mut g, p, "Forest", Zone::Hand);
        g.move_object(blocked, Zone::Battlefield);
        g.give_priority();
        finish(&mut g);
        assert!(tokens(&g, p).is_empty());
        g.state
            .roles
            .as_mut()
            .unwrap()
            .resolved_ability_modifications
            .clear();
        let lands = [
            put(&mut g, p, "Forest", Zone::Hand),
            put(&mut g, p, "Island", Zone::Hand),
        ];
        let moves = lands
            .iter()
            .map(|&id| g.entry_move(id, Zone::Battlefield, false))
            .collect();
        g.stage_entries(moves, None);
        g.give_priority();
        assert_eq!(g.state.pending_triggers.len(), 2);
        contracts::branch_contract(g.clone());
        g.move_object(saga, Zone::Exile);
        finish(&mut g);
        assert_eq!(tokens(&g, p).len(), 2);
        let movement = g.entry_move(saga, Zone::Battlefield, false);
        g.stage_entries(vec![movement], None);
        g.prepare_triggers();
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        act(&mut g, Action::FinishSearch);
        finish(&mut g);
        assert_ne!(source, g.object_ref(saga));
        let next = put(&mut g, p, "Swamp", Zone::Hand);
        g.move_object(next, Zone::Battlefield);
        g.give_priority();
        finish(&mut g);
        assert_eq!(tokens(&g, p).len(), 2);
    }
}

#[test]
fn down_valley_copied_saga_grant_and_additional_triggers() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let original = put(&mut g, p, "Down in the Valley", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 1, 2];
        cast(&mut g, original, None);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        act(&mut g, Action::FinishSearch);
        finish(&mut g);
        let copied = put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        let object = g.object_ref(copied);
        g.apply_temporary_copy(
            object,
            CopyRecipe::new(Card::from_name("Down in the Valley").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        g.effect_counter_by(
            copied,
            CounterKind::Lore,
            1,
            p,
            replacements::ResumeMode::Pending,
        );
        g.prepare_triggers();
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectSearch { .. })
        });
        act(&mut g, Action::FinishSearch);
        finish(&mut g);
        g.effect_counter_by(
            copied,
            CounterKind::Lore,
            1,
            p,
            replacements::ResumeMode::Pending,
        );
        g.prepare_triggers();
        finish(&mut g);
        let land = put(&mut g, p, "Forest", Zone::Hand);
        g.move_object(land, Zone::Battlefield);
        g.give_priority();
        finish(&mut g);
        assert_eq!(tokens(&g, p).len(), 1);
        // The grant remains on its exact recipient across a subsequent copy effect.
        g.apply_temporary_copy(
            object,
            CopyRecipe::new(Card::from_name("Bear Cub").unwrap()),
            CopyDeadline::EndOfTurn,
        );
        let timestamp = g.next_role_timestamp();
        g.state
            .roles
            .get_or_insert_with(Default::default)
            .creature_types
            .push(subtypes::CreatureTypesEffectView {
                recipient: object,
                source: object,
                types: &["Dwarf"],
                additional: false,
                timestamp,
            });
        put(&mut g, p, "Bifur, Melodic Rider", Zone::Battlefield);
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        put(&mut g, p, "Goldvein Pick", Zone::Battlefield);
        g.refresh_enduring_stories();
        assert_eq!(g.object_view(copied).additional_triggers, 1);
        let land2 = put(&mut g, p, "Island", Zone::Hand);
        g.move_object(land2, Zone::Battlefield);
        g.give_priority();
        finish(&mut g);
        assert_eq!(tokens(&g, p).len(), 3);
        assert!(
            g.state
                .temporary_abilities
                .as_ref()
                .unwrap()
                .iter()
                .any(|grant| grant.recipient == object)
        );
    }
}
