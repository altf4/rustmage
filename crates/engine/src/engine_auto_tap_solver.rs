//! An optional payment policy for independent, mutually exclusive source outputs.
//! The planner never walks a game tree. For one-unit sources, a group of mana
//! types can be paid iff it can draw on at least as many distinct sources as
//! there are required pips in that group, for every group (Hall's condition).
//! Generic pips additionally require sufficient total mana.
use super::{Mana, ManaCost};
use std::cmp::Reverse;
use std::collections::BTreeMap;

const TYPES: usize = 6;
const GROUPS: usize = 1 << TYPES;
const WUBRG: u8 = 0b11111;
const FOLLOWUP_LIMIT: usize = 3;

#[derive(Clone, Copy, Debug)]
pub(super) struct Mode {
    pub mana: Mana,
    pub pain: bool,
    pub sacrifice: bool,
}

#[derive(Clone, Debug)]
pub(super) struct Source {
    pub modes: Vec<Mode>,
    pub creature: bool,
    pub variable: bool,
    pub treasure: bool,
    /// Every offered mode can generate mana usable for a different purpose.
    pub followup_eligible: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Pools {
    pub eligible: Mana,
    pub treasure: Mana,
    pub followup: Mana,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct SpendBonus {
    /// Rewards each additional distinct color up to this many.
    pub distinct_colors: u8,
    /// Rewards spending at least one unit of Treasure-origin mana.
    pub treasure: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Solution {
    pub taps: Vec<(usize, usize)>,
    pub payment: Mana,
    pub prefer_treasure: bool,
}

fn mask(mana: Mana) -> u8 {
    mana.iter()
        .enumerate()
        .fold(0, |bits, (i, &n)| bits | (u8::from(n != 0) << i))
}

fn total(mana: Mana) -> u64 {
    mana.into_iter().map(u64::from).sum()
}

/// Slot zero counts all required/available mana, including generic costs.
/// Other slots count required pips in a group, or sources usable for that group.
#[derive(Clone)]
struct Totals([u64; GROUPS]);

impl Totals {
    fn demand(cost: ManaCost) -> Self {
        let mut v = [0; GROUPS];
        for group in 1usize..GROUPS {
            let color = group.trailing_zeros() as usize;
            v[group] = v[group & (group - 1)] + u64::from(cost.colored[color]);
        }
        v[0] = v[GROUPS - 1] + u64::from(cost.generic);
        Self(v)
    }

    fn combine(mut self, other: &Self) -> Self {
        for (a, b) in self.0.iter_mut().zip(other.0) {
            *a += b;
        }
        self
    }

    fn covers(&self, goal: &Self) -> bool {
        self.0.iter().zip(&goal.0).all(|(a, b)| a >= b)
    }

    fn add(&mut self, kinds: u8, quantity: u64) {
        if quantity == 0 {
            return;
        }
        self.0[0] += quantity;
        for (group, value) in self.0.iter_mut().enumerate().skip(1) {
            if group & usize::from(kinds) != 0 {
                *value += quantity;
            }
        }
    }

    fn take(&mut self, kinds: u8, quantity: u64) {
        self.0[0] -= quantity;
        for (group, value) in self.0.iter_mut().enumerate().skip(1) {
            if group & usize::from(kinds) != 0 {
                *value -= quantity;
            }
        }
    }
}

/// Every bucket contains only interchangeable unit reservations with the same
/// tap preference. Its cursor skips already-used sources once per entry. The
/// number of buckets is bounded by the six-bit mask and the fixed preferences.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Price {
    pain: bool,
    sacrifice: bool,
    variable: bool,
    extra_output: bool,
    creature: bool,
    ordinary: bool,
    unrestricted: bool,
}

struct Bucket {
    kinds: u8,
    price: Price,
    treasure: bool,
    entries: Vec<(usize, usize)>,
}

struct Inventory<'a> {
    sources: &'a [Source],
    kinds: Vec<u8>,
    by_color: [Vec<usize>; TYPES],
    buckets: Vec<Bucket>,
    bundles: Vec<usize>,
    capacity: Totals,
    pools: Pools,
}

impl<'a> Inventory<'a> {
    fn new(sources: &'a [Source], pools: Pools) -> Self {
        let mut result = Self {
            sources,
            kinds: Vec::with_capacity(sources.len()),
            by_color: Default::default(),
            buckets: Vec::new(),
            bundles: Vec::new(),
            capacity: Totals([0; GROUPS]),
            pools,
        };
        for (color, amount) in pools.eligible.into_iter().enumerate() {
            result.capacity.add(1 << color, u64::from(amount));
        }
        let mut keys = BTreeMap::new();
        let mut counts = [0u64; GROUPS];
        for (i, source) in sources.iter().enumerate() {
            let kinds = source.modes.iter().fold(0, |k, m| k | mask(m.mana));
            result.kinds.push(kinds);
            if kinds == 0 {
                continue;
            }
            counts[usize::from(kinds)] += 1;
            if source.modes.iter().any(|m| total(m.mana) > 1) {
                result.bundles.push(i);
            }
            let has_pain = source.modes.iter().any(|m| m.pain);
            for (j, mode) in source.modes.iter().enumerate() {
                for (color, &n) in mode.mana.iter().enumerate() {
                    if n == 0 {
                        continue;
                    }
                    let price = Price {
                        pain: mode.pain,
                        sacrifice: mode.sacrifice,
                        variable: source.variable,
                        extra_output: total(mode.mana) > 1,
                        creature: source.creature,
                        // A painland's damage-free {C} is useful for generic.
                        ordinary: !(color == 5 && has_pain && !mode.pain),
                        unrestricted: source.followup_eligible,
                    };
                    let bucket = *keys
                        .entry((color, kinds, price, source.treasure))
                        .or_insert_with(|| {
                            let index = result.buckets.len();
                            result.by_color[color].push(index);
                            result.buckets.push(Bucket {
                                kinds,
                                price,
                                treasure: source.treasure,
                                entries: Vec::new(),
                            });
                            index
                        });
                    result.buckets[bucket].entries.push((i, j));
                }
            }
        }
        for (kinds, count) in counts.into_iter().enumerate().skip(1) {
            result.capacity.add(kinds as u8, count);
        }
        result
    }
}

struct Draft<'a, 'b> {
    inventory: &'a Inventory<'b>,
    used: Vec<bool>,
    cursors: Vec<usize>,
    taps: Vec<(usize, usize)>,
    pool: [u64; TYPES],
    treasure_pool: [u64; TYPES],
    followup_pool: [u64; TYPES],
    payment: Mana,
    capacity: Totals,
    goal: Totals,
    cost: ManaCost,
    pain: usize,
    treasures: usize,
    variable: usize,
    creatures: usize,
    used_treasure: bool,
}

impl<'a, 'b> Draft<'a, 'b> {
    fn new(inventory: &'a Inventory<'b>, cost: ManaCost, reserve: &Totals) -> Self {
        Self {
            inventory,
            used: vec![false; inventory.sources.len()],
            cursors: vec![0; inventory.buckets.len()],
            taps: Vec::new(),
            pool: inventory.pools.eligible.map(u64::from),
            treasure_pool: inventory.pools.treasure.map(u64::from),
            followup_pool: inventory.pools.followup.map(u64::from),
            payment: [0; TYPES],
            capacity: inventory.capacity.clone(),
            goal: Totals::demand(cost).combine(reserve),
            cost,
            pain: 0,
            treasures: 0,
            variable: 0,
            creatures: 0,
            used_treasure: false,
        }
    }

    fn tap(&mut self, source: usize, mode: usize) {
        let item = &self.inventory.sources[source];
        let output = item.modes[mode];
        self.taps.push((source, mode));
        self.used[source] = true;
        self.capacity.take(self.inventory.kinds[source], 1);
        for (color, amount) in output.mana.into_iter().enumerate() {
            let amount = u64::from(amount);
            self.pool[color] += amount;
            self.capacity.add(1 << color, amount);
            if item.treasure {
                self.treasure_pool[color] += amount;
            }
            if item.followup_eligible {
                self.followup_pool[color] += amount;
            }
        }
        self.pain += usize::from(output.pain);
        self.treasures += usize::from(output.sacrifice);
        self.variable += usize::from(item.variable);
        self.creatures += usize::from(item.creature);
    }

    /// Prefer a concrete bundle that reduces the largest current shortfalls.
    /// Each source is visited once. Bundle modes are indivisible and remain a
    /// heuristic; the exact one-unit solver is untouched when already payable.
    fn fund_bundles(&mut self) -> bool {
        if self.capacity.covers(&self.goal) {
            return true;
        }
        for &source in &self.inventory.bundles {
            let kinds = usize::from(self.inventory.kinds[source]);
            let best = self.inventory.sources[source]
                .modes
                .iter()
                .enumerate()
                .filter_map(|(i, m)| {
                    if total(m.mana) <= 1 {
                        return None;
                    }
                    let output = Totals::demand(ManaCost {
                        colored: m.mana,
                        generic: 0,
                    });
                    let mut improvement = 0i64;
                    for (group, (&capacity, &needed)) in
                        self.capacity.0.iter().zip(&self.goal.0).enumerate()
                    {
                        let next = capacity - u64::from(group == 0 || group & kinds != 0)
                            + output.0[group];
                        improvement += needed.saturating_sub(capacity) as i64
                            - needed.saturating_sub(next) as i64;
                    }
                    (improvement > 0).then_some(((improvement, !m.pain, Reverse(i)), i))
                })
                .max_by_key(|&(score, _)| score);
            if let Some((_, mode)) = best {
                self.tap(source, mode);
                if self.capacity.covers(&self.goal) {
                    return true;
                }
            }
        }
        false
    }

    /// Sources connected to any tight group that is still owed in full are
    /// unavailable for diversion. A colored payment reduces demand in every
    /// group containing that color, so those groups need no reservation here.
    fn reserved(&self, paying: Option<usize>) -> u8 {
        let paid_bit = paying.map_or(0, |c| 1 << c);
        (1..GROUPS)
            .filter(|&s| s & paid_bit == 0 && self.capacity.0[s] == self.goal.0[s])
            .fold(0, |bits, s| bits | s as u8)
    }

    fn generic_room(&self, color: usize) -> u64 {
        (1..GROUPS)
            .filter(|s| s & (1 << color) != 0)
            .map(|s| self.capacity.0[s] - self.goal.0[s])
            .min()
            .unwrap_or(0)
    }

    fn spend(&mut self, color: usize, amount: u32, generic: bool) -> bool {
        let Some(paid) = self.payment[color].checked_add(amount) else {
            return false;
        };
        self.payment[color] = paid;
        let kind = 1 << color;
        if generic {
            self.cost.generic -= amount;
        } else {
            self.cost.colored[color] -= amount;
        }
        let amount = u64::from(amount);
        // The semantic executor can request Treasure lots of a given color when
        // the actual effect cares. Aggregate availability here is purpose-filtered.
        let treasure = self.treasure_pool[color].min(amount);
        self.used_treasure |= treasure > 0;
        self.treasure_pool[color] -= treasure;
        // Different lots of the same color may be spent. Only count the
        // unrestricted remainder guaranteed regardless of that allocation.
        self.followup_pool[color] = self.followup_pool[color].saturating_sub(amount);
        self.pool[color] -= amount;
        self.capacity.take(kind, amount);
        self.goal.take(if generic { 0 } else { kind }, amount);
        true
    }

    fn pick_source(
        &mut self,
        color: usize,
        reserved: u8,
        treasure_only: bool,
    ) -> Option<(usize, usize, Price, u32, u32)> {
        self.inventory.by_color[color]
            .iter()
            .filter_map(|&b| {
                let bucket = &self.inventory.buckets[b];
                if bucket.kinds & reserved != 0 || (treasure_only && !bucket.treasure) {
                    return None;
                }
                let index = &mut self.cursors[b];
                while *index < bucket.entries.len() && self.used[bucket.entries[*index].0] {
                    *index += 1;
                }
                let &(source, mode) = bucket.entries.get(*index)?;
                let lost = (0..5)
                    .filter(|&c| bucket.kinds & (1 << c) != 0 && self.capacity.0[1 << c] == 1)
                    .count() as u32;
                Some((source, mode, bucket.price, lost, bucket.kinds.count_ones()))
            })
            .min_by_key(|&(s, m, price, lost, flexible)| (price, lost, flexible, s, m))
    }

    /// Secure one Treasure-origin unit before ordinary floating mana would
    /// cover the entire cost. There are only six possible spend components.
    fn seek_treasure(&mut self) -> bool {
        for tapped in [false, true] {
            for color in 0..TYPES {
                let generic = self.cost.colored[color] == 0;
                if generic && self.cost.generic == 0 {
                    continue;
                }
                let reserved = self.reserved((!generic).then_some(color));
                if reserved & (1 << color) != 0 {
                    continue;
                }
                if tapped {
                    let Some((s, m, ..)) = self.pick_source(color, reserved, true) else {
                        continue;
                    };
                    self.tap(s, m);
                } else if self.treasure_pool[color] == 0 {
                    continue;
                }
                return self.spend(color, 1, generic);
            }
        }
        true
    }

    fn pay(&mut self, seek_treasure: bool) -> bool {
        if !self.fund_bundles() || seek_treasure && !self.seek_treasure() {
            return false;
        }
        while let Some(c) = (0..TYPES)
            .filter(|&c| self.cost.colored[c] > 0)
            .min_by_key(|&c| self.capacity.0[1 << c] - self.goal.0[1 << c])
        {
            if self.pool[c] == 0 {
                let reserved = self.reserved(Some(c));
                let Some((s, m, ..)) = self.pick_source(c, reserved, false) else {
                    return false;
                };
                self.tap(s, m);
            }
            let amount = self.pool[c].min(u64::from(self.cost.colored[c])) as u32;
            if !self.spend(c, amount, false) {
                return false;
            }
        }
        while self.cost.generic > 0 {
            let reserved = self.reserved(None);
            let paid = self.payment;
            let available = |c: &usize| reserved & (1 << c) == 0 && paid[*c] < u32::MAX;
            // Spend expiring pool mana in bulk. Keep the last accessible color
            // where possible; true colorless can never count toward a color bonus.
            let mut color = (0..TYPES)
                .filter(available)
                .filter(|&c| self.pool[c] > 0)
                .min_by_key(|&c| (c != 5 && self.capacity.0[1 << c] == 1, c != 5, c));
            if color.is_none() {
                let choice = (0..TYPES)
                    .filter(available)
                    .filter_map(|c| {
                        self.pick_source(c, reserved, false)
                            .map(|(s, m, p, l, f)| ((p, l, f, s, m, c), (s, m, c)))
                    })
                    .min_by_key(|&(rank, _)| rank);
                let Some((_, (s, m, c))) = choice else {
                    return false;
                };
                self.tap(s, m);
                color = Some(c);
            }
            let c = color.unwrap();
            let amount = self.pool[c]
                .min(self.generic_room(c))
                .min(u64::from(self.cost.generic))
                .min(u64::from(u32::MAX - self.payment[c])) as u32;
            if amount == 0 || !self.spend(c, amount, true) {
                return false;
            }
        }
        true
    }

    fn residual(&self) -> Totals {
        let mut capacity = self.capacity.clone();
        for c in 0..TYPES {
            capacity.take(1 << c, self.pool[c] - self.followup_pool[c]);
        }
        for (s, source) in self.inventory.sources.iter().enumerate() {
            if !source.followup_eligible && !self.used[s] && self.inventory.kinds[s] != 0 {
                capacity.take(self.inventory.kinds[s], 1);
            }
        }
        for &s in &self.inventory.bundles {
            if self.used[s] || !self.inventory.sources[s].followup_eligible {
                continue;
            }
            let modes = &self.inventory.sources[s].modes;
            if modes.iter().all(|m| m.mana == modes[0].mana) {
                capacity.take(self.inventory.kinds[s], 1);
                for (c, n) in modes[0].mana.into_iter().enumerate() {
                    capacity.add(1 << c, u64::from(n));
                }
            }
        }
        capacity
    }
}

/// Substitute colored requirements for generic pips of the same payment.
fn require_colors(mut cost: ManaCost, colors: u8) -> Option<ManaCost> {
    for (c, n) in cost.colored[..5].iter_mut().enumerate() {
        if colors & (1 << c) != 0 && *n == 0 {
            cost.generic = cost.generic.checked_sub(1)?;
            *n = 1;
        }
    }
    Some(cost)
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Score {
    colors: u8,
    treasure_benefit: bool,
    preferred_followup: bool,
    pain: Reverse<usize>,
    treasures: Reverse<usize>,
    followups: usize,
    variable: Reverse<usize>,
    creatures: Reverse<usize>,
    taps: Reverse<usize>,
    colors_left: usize,
}

pub(super) fn solve(
    cost: ManaCost,
    pools: Pools,
    sources: &[Source],
    preferred_followup: Option<ManaCost>,
    followups: &[ManaCost],
    bonus: SpendBonus,
) -> Option<Solution> {
    let inventory = Inventory::new(sources, pools);
    // Reuse the same 64-counter test for follow-ups. The fixed shortlist bounds
    // alternative tap proposals; all observed costs can still score the result.
    let mut reservations = vec![ManaCost::ZERO];
    for &next in preferred_followup.iter().chain(followups) {
        if reservations.len() > FOLLOWUP_LIMIT {
            break;
        }
        if !reservations.contains(&next) {
            reservations.push(next);
        }
    }
    let preferred = preferred_followup.map(Totals::demand);
    let mut best: Option<(Score, Solution)> = None;
    let present_colors = mask(cost.colored) & WUBRG;
    let target = bonus.distinct_colors.min(5).min(
        total(cost.colored)
            .saturating_add(u64::from(cost.generic))
            .min(5) as u8,
    );
    let available_colors = (0..5).fold(0, |k, c| {
        k | (u8::from(inventory.capacity.0[1 << c] != 0) << c)
    });
    let mut targets = vec![cost];
    if target > present_colors.count_ones() as u8 {
        for colors in 1..=WUBRG {
            if colors & !available_colors != 0
                || colors & present_colors != present_colors
                || colors.count_ones() > u32::from(target)
            {
                continue;
            }
            if let Some(t) = require_colors(cost, colors)
                && !targets.contains(&t)
            {
                targets.push(t);
            }
        }
    }
    // Try the highest scoring color targets first. The original cost remains
    // available when no bonus can be earned.
    targets.sort_by_key(|c| Reverse((mask(c.colored) & WUBRG).count_ones()));
    for target_cost in targets {
        let required_colors = (mask(target_cost.colored) & WUBRG).count_ones() as u8;
        if best
            .as_ref()
            .is_some_and(|(s, _)| s.colors > required_colors && required_colors < target)
        {
            break;
        }
        for &reserve in &reservations {
            let mut draft = Draft::new(&inventory, target_cost, &Totals::demand(reserve));
            if !draft.pay(bonus.treasure) {
                continue;
            }
            let residual = draft.residual();
            let earned_treasure = bonus.treasure && draft.used_treasure;
            let score = Score {
                colors: ((mask(draft.payment) & WUBRG).count_ones() as u8).min(target),
                treasure_benefit: earned_treasure,
                preferred_followup: preferred.as_ref().is_some_and(|d| residual.covers(d)),
                pain: Reverse(draft.pain),
                treasures: Reverse(draft.treasures),
                followups: followups
                    .iter()
                    .filter(|&&c| residual.covers(&Totals::demand(c)))
                    .count(),
                variable: Reverse(draft.variable),
                creatures: Reverse(draft.creatures),
                taps: Reverse(draft.taps.len()),
                colors_left: (0..5).filter(|&c| residual.0[1 << c] > 0).count(),
            };
            if best.as_ref().is_none_or(|(old, _)| score > *old) {
                best = Some((
                    score,
                    Solution {
                        taps: draft.taps,
                        payment: draft.payment,
                        prefer_treasure: earned_treasure,
                    },
                ));
            }
        }
    }
    best.map(|(_, solution)| solution)
}
