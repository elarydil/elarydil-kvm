use crate::model::*;
use std::collections::HashMap;
#[derive(Clone, Debug, Default)]
struct State {
    armed: bool,
    opposite_since: Option<u64>,
    candidate: Option<u64>,
    last: HashMap<String, bool>,
    edges: HashMap<String, u64>,
    last_fire: Option<u64>,
}
#[derive(Default)]
pub struct Engine {
    states: HashMap<String, State>,
    pub grace_until: u64,
    last_now: u64,
}
#[derive(Debug, PartialEq)]
pub struct Decision {
    pub rule_id: String,
    pub profile_id: String,
    pub dry_run: bool,
    pub actions: Vec<Action>,
}
impl PartialEq for Action {
    fn eq(&self, b: &Self) -> bool {
        self.id == b.id
            && self.monitor_id == b.monitor_id
            && self.input_value == b.input_value
            && self.vcp_code == b.vcp_code
    }
}
impl Engine {
    pub fn reset(&mut self, now: u64, grace: u64) {
        self.states.clear();
        self.last_now = now;
        self.grace_until = now.saturating_add(grace)
    }
    /// Arm only from the stable USB baseline explicitly captured by the wizard.
    /// Priming never emits an action; the next fresh opposite-to-target edge can fire.
    pub fn prime(&mut self, config: &Config, present: &HashMap<String, bool>, now: u64) {
        self.reset(now, 0);
        self.tick(config, present, now);
        for profile in config.profiles.iter().filter(|p| p.enabled) {
            for rule in profile.rules.iter().filter(|r| r.enabled) {
                if rule.device_ids.iter().any(|id| !present.contains_key(id)) {
                    continue;
                }
                let needed = match rule.mode.as_str() {
                    "all" => rule.device_ids.len(),
                    "any" => 1,
                    _ => rule.minimum_count,
                };
                let target = rule.trigger == "connect";
                let count = rule
                    .device_ids
                    .iter()
                    .filter(|id| present[*id] == target)
                    .count();
                if let Some(state) = self.states.get_mut(&rule.id) {
                    state.armed = count < needed;
                    state.candidate = None;
                    state.edges.clear();
                }
            }
        }
    }
    pub fn tick(
        &mut self,
        config: &Config,
        present: &HashMap<String, bool>,
        now: u64,
    ) -> Vec<Decision> {
        let mut out = vec![];
        if now < self.last_now {
            return out;
        }
        self.last_now = now;
        for p in config.profiles.iter().filter(|p| p.enabled) {
            for r in p.rules.iter().filter(|r| r.enabled) {
                let s = self.states.entry(r.id.clone()).or_default();
                let target = r.trigger == "connect";
                if r.device_ids.iter().any(|d| !present.contains_key(d)) {
                    s.armed = false;
                    s.candidate = None;
                    s.opposite_since = None;
                    continue;
                }
                let mut changed = false;
                for d in &r.device_ids {
                    let v = present[d];
                    if let Some(old) = s.last.insert(d.clone(), v) {
                        if old != v {
                            changed = true;
                            if v == target {
                                s.edges.insert(d.clone(), now);
                            } else {
                                s.edges.remove(d);
                            }
                        }
                    }
                }
                let needed = match r.mode.as_str() {
                    "all" => r.device_ids.len(),
                    "any" => 1,
                    _ => r.minimum_count,
                };
                let count = r
                    .device_ids
                    .iter()
                    .filter(|d| present[*d] == target)
                    .count();
                let opposite = count < needed;
                if now < self.grace_until {
                    s.armed = false;
                    s.candidate = None;
                    s.opposite_since = None;
                    s.edges.clear();
                    continue;
                }
                if !s.armed {
                    if opposite {
                        let since = s.opposite_since.get_or_insert(now);
                        if now.saturating_sub(*since) >= r.rearm_ms {
                            s.armed = true;
                            s.edges.clear();
                        }
                    } else {
                        s.opposite_since = None
                    }
                    continue;
                }
                if count < needed {
                    s.candidate = None;
                    continue;
                }
                let mut times: Vec<_> = r
                    .device_ids
                    .iter()
                    .filter(|d| present[*d] == target)
                    .filter_map(|d| s.edges.get(d))
                    .copied()
                    .collect();
                times.sort_unstable();
                let coincident = if let Some(w) = r.coincidence_window_ms {
                    times.len() >= needed
                        && times[times.len() - 1].saturating_sub(times[times.len() - needed]) <= w
                } else {
                    true
                };
                if !coincident {
                    s.candidate = None;
                    continue;
                }
                if s.candidate.is_none() && !changed {
                    continue;
                }
                let since = *s.candidate.get_or_insert(now);
                if now.saturating_sub(since) < r.debounce_ms {
                    continue;
                }
                s.armed = false;
                s.candidate = None;
                s.opposite_since = None;
                s.edges.clear();
                if s.last_fire
                    .is_some_and(|last| now.saturating_sub(last) < r.cooldown_ms)
                {
                    continue;
                }
                s.last_fire = Some(now);
                out.push(Decision {
                    rule_id: r.id.clone(),
                    profile_id: p.id.clone(),
                    dry_run: r.dry_run,
                    actions: r.actions.clone(),
                });
            }
        }
        out
    }
}
