//! Immutable as-though casting permissions, indexed with battlefield statics.
use super::*;
impl Game {
    pub(super) fn cast_timing_granted(&self, actor: Player, chosen: Card) -> bool {
        self.state.static_sources.iter().any(|&source| {
            let object = &self.state.objects[source as usize];
            object.zone == Zone::Battlefield
                && self.printed_abilities_available(source)
                && object.controller == actor
                && object
                    .card
                    .cast_timing_grants()
                    .iter()
                    .any(|filter| filter.matches(chosen))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::tests::{scenario, spawn};
    const WHIRL: &str = "Whirlwing Stormbrood // Dynamic Soar";
    fn setup(p: Player) -> Game {
        let mut g = scenario();
        g.state.active = p.other();
        g.state.priority = p;
        g.state.players[p.index()].mana = [30; 6];
        spawn(
            &mut g,
            p,
            Card::from_name("Bear Cub").unwrap(),
            Zone::Battlefield,
        );
        g
    }
    fn act(g: &mut Game, a: Action) {
        let actor = g.actor().unwrap();
        let saved = g.snapshot();
        let revision = g.revision();
        let index = g
            .legal_actions()
            .iter()
            .position(|x| *x == a)
            .expect("legal action");
        assert_eq!(g.apply(actor.other(), a.clone()), Err(Error::WrongActor));
        assert_eq!(g.snapshot(), saved);
        let mut child = {
            let parent = g.clone();
            let c = parent.clone();
            drop(parent);
            c
        };
        child.apply_index(actor, revision, index).unwrap();
        g.apply(actor, a.clone()).unwrap();
        assert_eq!(g.snapshot(), child.snapshot());
        for viewer in [Player::P0, Player::P1] {
            assert_eq!(
                serde_json::to_value(g.observe(viewer)).unwrap(),
                serde_json::to_value(child.observe(viewer)).unwrap()
            );
        }
        let done = g.snapshot();
        g.restore(&saved).unwrap();
        assert_eq!(
            g.apply_index(actor, revision, index),
            Err(Error::StaleRevision)
        );
        g.apply(actor, a).unwrap();
        assert_eq!(g.snapshot(), done);
    }
    fn choose(g: &mut Game) {
        let actions = g.legal_actions();
        let a = actions
            .iter()
            .find(|a| matches!(a, Action::FinishTargets | Action::FinishSearch))
            .or_else(|| actions.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        act(g, a);
    }
    fn announce(g: &mut Game, a: Action, id: CardId) {
        act(g, a);
        for _ in 0..100 {
            if g.state.pending == Pending::Priority && g.state.stack.iter().any(|s| s.card == id) {
                return;
            }
            choose(g);
        }
        panic!("announcement did not complete");
    }
    fn settle(g: &mut Game) {
        for _ in 0..200 {
            if g.state.pending == Pending::Priority
                && g.state.stack.is_empty()
                && g.state.resolving.is_none()
            {
                return;
            }
            choose(g);
        }
        panic!("resolution did not complete");
    }
    fn allowed(g: &Game, a: Action) -> bool {
        g.legal_actions().contains(&a)
    }
    fn denied(g: &mut Game, a: Action) {
        assert!(!allowed(g, a.clone()));
        let before = g.snapshot();
        assert!(g.apply(g.actor().unwrap(), a).is_err());
        assert_eq!(before, g.snapshot());
    }
    #[test]
    fn printed_flash_does_not_leak_to_sorcery_omen_but_paid_source_grants_chosen_spells() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let first = spawn(&mut g, p, Card::from_name(WHIRL).unwrap(), Zone::Hand);
            denied(&mut g, Action::AnnounceOmen { card: first });
            announce(&mut g, Action::AnnounceCast { card: first }, first);
            settle(&mut g);
            assert_eq!(g.state.objects[first as usize].zone, Zone::Battlefield);
            assert!(g.keywords(first).contains(Keywords::FLASH));
            assert!(g.keywords(first).contains(Keywords::FLYING));
            g.state.priority = p;
            let second = spawn(&mut g, p, Card::from_name(WHIRL).unwrap(), Zone::Hand);
            let bear = spawn(&mut g, p, Card::from_name("Bear Cub").unwrap(), Zone::Hand);
            denied(&mut g, Action::AnnounceCast { card: bear });
            let dragon = spawn(
                &mut g,
                p,
                Card::from_name("Dirgur Island Dragon // Skimming Strike").unwrap(),
                Zone::Hand,
            );
            assert!(allowed(&g, Action::AnnounceCast { card: dragon }));
            assert!(!g.keywords(dragon).contains(Keywords::FLASH));
            let target = g
                .state
                .objects
                .iter()
                .position(|o| {
                    o.owner == p
                        && o.zone == Zone::Battlefield
                        && o.card == Card::from_name("Bear Cub").unwrap()
                })
                .unwrap();
            let count = g.state.objects.len();
            announce(&mut g, Action::AnnounceOmen { card: second }, second);
            assert_eq!(
                g.state.stack.last().unwrap().source_card,
                Card::from_name(WHIRL).unwrap().omen_spell().unwrap()
            );
            settle(&mut g);
            assert_eq!(g.state.objects[target].counters.plus_one, 3);
            assert_eq!(g.state.objects[second as usize].zone, Zone::Library);
            assert_eq!(
                g.state.players[p.index()]
                    .library
                    .iter()
                    .filter(|&&id| id == second)
                    .count(),
                1
            );
            assert_eq!(g.state.objects.len(), count);
            assert!(g.state.adventures.is_none());
        }
    }
    #[test]
    fn source_control_zone_funding_and_land_special_actions_remain_enforced() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let source = spawn(
                &mut g,
                p.other(),
                Card::from_name(WHIRL).unwrap(),
                Zone::Battlefield,
            );
            let spell = spawn(&mut g, p, Card::from_name(WHIRL).unwrap(), Zone::Hand);
            denied(&mut g, Action::AnnounceOmen { card: spell });
            g.move_object(source, Zone::Graveyard);
            let own = spawn(
                &mut g,
                p,
                Card::from_name(WHIRL).unwrap(),
                Zone::Battlefield,
            );
            assert!(allowed(&g, Action::AnnounceOmen { card: spell }));
            let saved = g.snapshot();
            g.move_object(own, Zone::Graveyard);
            denied(&mut g, Action::AnnounceOmen { card: spell });
            g.restore(&saved).unwrap();
            assert!(allowed(&g, Action::AnnounceOmen { card: spell }));
            g.state.players[p.index()].mana = [0; 6];
            denied(&mut g, Action::AnnounceOmen { card: spell });
            g.state.players[p.index()].mana = [30; 6];
            let land = spawn(&mut g, p, Card::Forest, Zone::Hand);
            denied(&mut g, Action::PlayLand { card: land });
            let plot = spawn(
                &mut g,
                p,
                Card::from_name("Stingerback Terror").unwrap(),
                Zone::Hand,
            );
            assert!(allowed(&g, Action::AnnounceCast { card: plot }));
            denied(&mut g, Action::AnnouncePlot { card: plot });
        }
    }
    #[test]
    fn plotted_explicit_restriction_and_already_paid_spells_keep_their_rules() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            let source = spawn(
                &mut g,
                p,
                Card::from_name(WHIRL).unwrap(),
                Zone::Battlefield,
            );
            let terror = spawn(
                &mut g,
                p,
                Card::from_name("Stingerback Terror").unwrap(),
                Zone::Hand,
            );
            g.state.active = p;
            act(&mut g, Action::AnnouncePlot { card: terror });
            for _ in 0..40 {
                if g.state.pending == Pending::Priority {
                    break;
                }
                choose(&mut g);
            }
            assert_eq!(g.state.objects[terror as usize].zone, Zone::Exile);
            g.state.turn += 1;
            g.state.active = p.other();
            g.state.priority = p;
            denied(&mut g, Action::AnnouncePlotted { card: terror });
            let omen = spawn(&mut g, p, Card::from_name(WHIRL).unwrap(), Zone::Hand);
            announce(&mut g, Action::AnnounceOmen { card: omen }, omen);
            g.move_object(source, Zone::Graveyard);
            settle(&mut g);
            assert_eq!(g.state.objects[omen as usize].zone, Zone::Library);
            let bear = g
                .state
                .objects
                .iter()
                .find(|o| {
                    o.owner == p
                        && o.zone == Zone::Battlefield
                        && o.card == Card::from_name("Bear Cub").unwrap()
                })
                .unwrap();
            assert_eq!(bear.counters.plus_one, 3);
        }
    }
    #[test]
    fn chosen_sorcery_adventure_gets_timing_but_normal_exile_face_does_not() {
        for p in [Player::P0, Player::P1] {
            let mut g = setup(p);
            spawn(
                &mut g,
                p,
                Card::from_name(WHIRL).unwrap(),
                Zone::Battlefield,
            );
            let bilbo = spawn(
                &mut g,
                p,
                Card::from_name("Bilbo Baggins, Burglar // Take a Glance").unwrap(),
                Zone::Hand,
            );
            denied(&mut g, Action::AnnounceCast { card: bilbo });
            announce(&mut g, Action::AnnounceAdventure { card: bilbo }, bilbo);
            settle(&mut g);
            assert_eq!(g.state.objects[bilbo as usize].zone, Zone::Exile);
            assert!(g.can_play_exiled(p, bilbo));
            g.state.priority = p;
            denied(&mut g, Action::AnnounceExile { card: bilbo });
        }
    }
}
