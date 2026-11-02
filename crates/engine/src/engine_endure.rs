//! Exact-source resolving endure choices; the continuation is part of Pending.
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl Game {
    /// CR701.63: the enduring permanent's controller makes this choice. The
    /// packet already freezes its actual battlefield controller on departure.
    pub(super) fn start_fixed_endure(
        &mut self,
        item: &SpellView,
        amount: u16,
        token: Card,
    ) -> bool {
        if amount == 0 {
            return false;
        }
        let source = item.source.expect("endure requires an object source");
        let live = self.on_battlefield(source.id) && self.object_ref(source.id) == source.object;
        let player = if live {
            self.state.objects[source.id as usize].controller
        } else {
            source.facts.controller
        };
        if live && !replacements::counters_prohibited(self, source.object) {
            self.state.pending = Pending::Endure {
                player,
                source: source.object,
                amount,
                token,
            };
            return true;
        }
        // A prohibited placement cannot be chosen instead of creating the
        // Spirit. Nor may a departed source bind a later incarnation.
        self.create_tokens(player, token, 1, false);
        false
    }

    pub(super) fn choose_endure(&mut self, put_counters: bool) {
        let Pending::Endure {
            player,
            source,
            amount,
            token,
        } = self.state.pending
        else {
            unreachable!()
        };
        // No action other than the actor's choice or concession can interleave
        // while resolution is suspended. The source was validated at creation.
        assert!(self.on_battlefield(source.card) && self.object_ref(source.card) == source);
        if put_counters {
            self.effect_counter(
                source.card,
                CounterKind::PlusOne,
                u64::from(amount),
                replacements::ResumeMode::Program,
            );
        } else {
            self.create_tokens(player, token, 1, false);
        }
        self.run_program();
    }
}
