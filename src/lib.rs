//! # agent-fermata
//!
//! Strategic pauses for agent systems.
//!
//! In music notation, a *fermata* (𝄐) is a symbol that means "hold this note
//! longer than its written duration — as long as it feels right." It's not a
//! mistake or a delay; it's a deliberate, expressive choice. This crate gives
//! agents the same ability: to pause strategically, breathe, wait for the right
//! moment, and resume when conditions are right.

use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// PauseReason
// ---------------------------------------------------------------------------

/// Why an agent is pausing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PauseReason {
    /// Waiting for external input (user message, API response, etc.).
    WaitingForInput,
    /// Performing internal processing (thinking, computing, etc.).
    Processing,
    /// Pausing for strategic timing (rate limiting, backoff, scheduling).
    StrategicTiming,
    /// Mandatory system health pause (cooldown, resource recovery).
    SystemHealth,
    /// Waiting for a dependency or prerequisite.
    WaitingForDependency,
    /// Deliberate pause for observation (monitoring, watching for changes).
    Observation,
    /// User-requested pause.
    UserRequested,
}

impl fmt::Display for PauseReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PauseReason::WaitingForInput => write!(f, "waiting for input"),
            PauseReason::Processing => write!(f, "processing"),
            PauseReason::StrategicTiming => write!(f, "strategic timing"),
            PauseReason::SystemHealth => write!(f, "system health"),
            PauseReason::WaitingForDependency => write!(f, "waiting for dependency"),
            PauseReason::Observation => write!(f, "observation"),
            PauseReason::UserRequested => write!(f, "user requested"),
        }
    }
}

// ---------------------------------------------------------------------------
// ResumeCondition
// ---------------------------------------------------------------------------

/// What triggers continuation after a pause.
#[derive(Debug, Clone, PartialEq)]
pub enum ResumeCondition {
    /// Resume after a fixed duration.
    AfterDuration(Duration),
    /// Resume when a specific condition becomes true (described by string).
    WhenConditionMet(String),
    /// Resume when external input arrives.
    OnInput,
    /// Resume when a dependency is satisfied.
    OnDependencyReady(String),
    /// Resume when a counter reaches a threshold.
    OnCountReached { current: u64, target: u64 },
    /// Resume immediately (no real pause).
    Immediate,
    /// Never auto-resume; must be explicitly resumed.
    Manual,
}

impl ResumeCondition {
    /// Check if the resume condition is satisfied.
    pub fn is_satisfied(&self) -> bool {
        match self {
            ResumeCondition::Immediate => true,
            ResumeCondition::Manual => false,
            ResumeCondition::OnCountReached { current, target } => current >= target,
            _ => false, // Others need external checks
        }
    }

    /// For `AfterDuration`, check if the pause started at `start` has elapsed.
    pub fn is_duration_elapsed(&self, start: Instant) -> bool {
        match self {
            ResumeCondition::AfterDuration(d) => start.elapsed() >= *d,
            _ => false,
        }
    }
}

// ---------------------------------------------------------------------------
// PauseEffect
// ---------------------------------------------------------------------------

/// How pauses affect subsequent performance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PauseEffect {
    /// The pause refreshed the agent; performance improves.
    Refreshed {
        /// Quality improvement factor (> 1.0 means better).
        quality_boost: f64,
        /// Speed improvement factor (> 1.0 means faster).
        speed_boost: f64,
    },
    /// The pause was neutral; no significant effect.
    Neutral,
    /// The pause was too long; momentum was lost.
    MomentumLoss {
        /// Warmup time needed to get back to speed.
        warmup_duration: Duration,
    },
    /// The pause caused a timeout or staleness.
    Stale {
        /// How stale the context is (0.0 = fresh, 1.0 = completely stale).
        staleness: f64,
    },
}

impl PauseEffect {
    /// Compute a combined performance score (0.0–2.0 scale).
    pub fn performance_score(&self) -> f64 {
        match self {
            PauseEffect::Refreshed { quality_boost, speed_boost } => {
                (quality_boost + speed_boost) / 2.0
            }
            PauseEffect::Neutral => 1.0,
            PauseEffect::MomentumLoss { warmup_duration } => {
                let secs = warmup_duration.as_secs_f64();
                (1.0 - secs / 60.0).max(0.5)
            }
            PauseEffect::Stale { staleness } => {
                1.0 - *staleness
            }
        }
    }

    /// Is this a positive effect?
    pub fn is_positive(&self) -> bool {
        self.performance_score() > 1.0
    }
}

// ---------------------------------------------------------------------------
// Fermata
// ---------------------------------------------------------------------------

/// A pause point with configurable hold duration.
///
/// Like the musical fermata symbol, this represents a deliberate pause that
/// can last longer than "written" — the agent holds until it feels right.
#[derive(Debug, Clone)]
pub struct Fermata {
    /// Unique identifier for this pause point.
    id: String,
    /// Why the agent is pausing.
    reason: PauseReason,
    /// Minimum hold duration.
    min_hold: Duration,
    /// Maximum hold duration (None = unlimited).
    max_hold: Option<Duration>,
    /// What triggers resumption.
    resume_condition: ResumeCondition,
    /// Effect of this pause on performance.
    effect: PauseEffect,
    /// Priority (higher = more important pause).
    priority: u32,
    /// When the pause started.
    started_at: Option<Instant>,
    /// Whether this pause is currently active.
    active: bool,
    /// Metadata / context.
    metadata: HashMap<String, String>,
}

impl Fermata {
    /// Create a new fermata with a minimum hold duration.
    pub fn new(id: impl Into<String>, reason: PauseReason, min_hold: Duration) -> Self {
        Self {
            id: id.into(),
            reason,
            min_hold,
            max_hold: None,
            resume_condition: ResumeCondition::AfterDuration(min_hold),
            effect: PauseEffect::Neutral,
            priority: 5,
            started_at: None,
            active: false,
            metadata: HashMap::new(),
        }
    }

    /// Create a brief strategic pause.
    pub fn brief(id: impl Into<String>, reason: PauseReason) -> Self {
        Self::new(id, reason, Duration::from_millis(100))
    }

    /// Create a long strategic pause.
    pub fn long(id: impl Into<String>, reason: PauseReason) -> Self {
        Self::new(id, reason, Duration::from_secs(30))
    }

    /// Set the maximum hold duration.
    pub fn with_max_hold(mut self, max: Duration) -> Self {
        self.max_hold = Some(max);
        self
    }

    /// Set the resume condition.
    pub fn with_resume_condition(mut self, condition: ResumeCondition) -> Self {
        self.resume_condition = condition;
        self
    }

    /// Set the pause effect.
    pub fn with_effect(mut self, effect: PauseEffect) -> Self {
        self.effect = effect;
        self
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Start the pause.
    pub fn start(&mut self) {
        self.started_at = Some(Instant::now());
        self.active = true;
    }

    /// Check if the pause should be over.
    pub fn should_resume(&self) -> bool {
        if !self.active {
            return true;
        }
        let start = match self.started_at {
            Some(s) => s,
            None => return true,
        };
        let elapsed = start.elapsed();

        // Check minimum hold
        if elapsed < self.min_hold {
            return false;
        }

        // Check maximum hold
        if let Some(max) = self.max_hold {
            if elapsed >= max {
                return true;
            }
        }

        // Check resume condition
        match &self.resume_condition {
            ResumeCondition::AfterDuration(d) => elapsed >= *d,
            ResumeCondition::Immediate => true,
            ResumeCondition::Manual => false,
            ResumeCondition::OnCountReached { current, target } => current >= target,
            _ => elapsed >= self.min_hold, // default: resume after min_hold
        }
    }

    /// Force resume the pause.
    pub fn force_resume(&mut self) -> PauseEffect {
        self.active = false;
        self.effect.clone()
    }

    /// Get elapsed time since pause started.
    pub fn elapsed(&self) -> Option<Duration> {
        self.started_at.map(|s| s.elapsed())
    }

    /// Is the pause currently active?
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// The pause ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The reason for the pause.
    pub fn reason(&self) -> PauseReason {
        self.reason
    }

    /// Minimum hold duration.
    pub fn min_hold(&self) -> Duration {
        self.min_hold
    }

    /// Maximum hold duration.
    pub fn max_hold(&self) -> Option<Duration> {
        self.max_hold
    }

    /// The resume condition.
    pub fn resume_condition(&self) -> &ResumeCondition {
        &self.resume_condition
    }

    /// The pause effect.
    pub fn effect(&self) -> &PauseEffect {
        &self.effect
    }

    /// Priority.
    pub fn priority(&self) -> u32 {
        self.priority
    }

    /// Metadata.
    pub fn metadata(&self) -> &HashMap<String, String> {
        &self.metadata
    }
}

// ---------------------------------------------------------------------------
// PauseDetector
// ---------------------------------------------------------------------------

/// Identifies natural pause points in a sequence of events.
///
/// `PauseDetector` looks for lulls, boundaries, and transitions in agent
/// activity where a strategic pause would be natural and beneficial.
#[derive(Debug, Clone)]
pub struct PauseDetector {
    /// Minimum gap between events to be considered a pause opportunity.
    gap_threshold: Duration,
    /// Event types that signal a natural boundary.
    boundary_events: Vec<String>,
    /// Maximum events to track.
    window_size: usize,
    /// Recent event timestamps.
    recent_events: Vec<(String, Instant)>,
}

impl PauseDetector {
    /// Create a new pause detector.
    pub fn new(gap_threshold: Duration) -> Self {
        Self {
            gap_threshold,
            boundary_events: Vec::new(),
            window_size: 100,
            recent_events: Vec::new(),
        }
    }

    /// Add a boundary event type.
    pub fn add_boundary_event(&mut self, event_type: impl Into<String>) {
        self.boundary_events.push(event_type.into());
    }

    /// Record an event.
    pub fn record_event(&mut self, event_type: impl Into<String>) {
        self.recent_events.push((event_type.into(), Instant::now()));
        if self.recent_events.len() > self.window_size {
            self.recent_events.remove(0);
        }
    }

    /// Check if now is a good time to pause.
    pub fn is_good_pause_point(&self) -> bool {
        if self.recent_events.is_empty() {
            return true; // No recent activity = good time to pause
        }
        let last = self.recent_events.last().unwrap();
        let elapsed = last.1.elapsed();
        // Good pause point if there's been a gap or the last event is a boundary
        let is_gap = elapsed >= self.gap_threshold;
        let is_boundary = self.boundary_events.contains(&last.0);
        is_gap || is_boundary
    }

    /// Get the time since the last event.
    pub fn time_since_last_event(&self) -> Option<Duration> {
        self.recent_events.last().map(|(_, t)| t.elapsed())
    }

    /// Get the average gap between recent events.
    pub fn average_gap(&self) -> Option<Duration> {
        if self.recent_events.len() < 2 {
            return None;
        }
        let total: Duration = self.recent_events
            .windows(2)
            .map(|w| {
                let diff = w[1].1.duration_since(w[0].1);
                w[1].1 - w[0].1
            })
            .fold(Duration::ZERO, |acc, d| acc + d);
        let count = self.recent_events.len() - 1;
        Some(total / count as u32)
    }

    /// Number of recent events tracked.
    pub fn event_count(&self) -> usize {
        self.recent_events.len()
    }

    /// Clear all tracked events.
    pub fn clear(&mut self) {
        self.recent_events.clear();
    }
}

// ---------------------------------------------------------------------------
// BreathMark
// ---------------------------------------------------------------------------

/// A mandatory pause for system health.
///
/// In music, a breath mark (') tells the performer to take a quick breath.
/// In agent systems, a `BreathMark` ensures the agent periodically pauses
/// for health checks, resource recovery, or cooldowns.
#[derive(Debug, Clone)]
pub struct BreathMark {
    /// How often the breath should be taken.
    interval: Duration,
    /// How long the breath lasts.
    breath_duration: Duration,
    /// When the last breath was taken.
    last_breath: Option<Instant>,
    /// Maximum operations between breaths.
    max_ops_between: u64,
    /// Operations since last breath.
    ops_since_breath: u64,
    /// Whether the breath mark is enabled.
    enabled: bool,
}

impl BreathMark {
    /// Create a new breath mark.
    pub fn new(interval: Duration, breath_duration: Duration) -> Self {
        Self {
            interval,
            breath_duration,
            last_breath: None,
            max_ops_between: 1000,
            ops_since_breath: 0,
            enabled: true,
        }
    }

    /// Create a default breath mark (every 60s, 1s breath).
    pub fn default_breath() -> Self {
        Self::new(Duration::from_secs(60), Duration::from_secs(1))
    }

    /// Set max operations between breaths.
    pub fn with_max_ops(mut self, max: u64) -> Self {
        self.max_ops_between = max;
        self
    }

    /// Record an operation.
    pub fn record_op(&mut self) {
        self.ops_since_breath += 1;
    }

    /// Check if a breath is needed.
    pub fn needs_breath(&self) -> bool {
        if !self.enabled {
            return false;
        }
        // Time-based check
        let time_for_breath = match self.last_breath {
            Some(last) => last.elapsed() >= self.interval,
            None => true,
        };
        // Op-count check
        let ops_for_breath = self.ops_since_breath >= self.max_ops_between;
        time_for_breath || ops_for_breath
    }

    /// Take a breath (record that a pause happened).
    pub fn take_breath(&mut self) {
        self.last_breath = Some(Instant::now());
        self.ops_since_breath = 0;
    }

    /// How long the breath should last.
    pub fn breath_duration(&self) -> Duration {
        self.breath_duration
    }

    /// The breath interval.
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Operations since last breath.
    pub fn ops_since_breath(&self) -> u64 {
        self.ops_since_breath
    }

    /// Enable or disable.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Is the breath mark enabled?
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- PauseReason tests ---

    #[test]
    fn test_pause_reason_display() {
        assert_eq!(format!("{}", PauseReason::WaitingForInput), "waiting for input");
        assert_eq!(format!("{}", PauseReason::Processing), "processing");
        assert_eq!(format!("{}", PauseReason::StrategicTiming), "strategic timing");
        assert_eq!(format!("{}", PauseReason::SystemHealth), "system health");
    }

    #[test]
    fn test_pause_reason_equality() {
        assert_eq!(PauseReason::WaitingForInput, PauseReason::WaitingForInput);
        assert_ne!(PauseReason::WaitingForInput, PauseReason::Processing);
    }

    // --- ResumeCondition tests ---

    #[test]
    fn test_resume_immediate() {
        let cond = ResumeCondition::Immediate;
        assert!(cond.is_satisfied());
    }

    #[test]
    fn test_resume_manual() {
        let cond = ResumeCondition::Manual;
        assert!(!cond.is_satisfied());
    }

    #[test]
    fn test_resume_on_count() {
        let cond = ResumeCondition::OnCountReached { current: 5, target: 10 };
        assert!(!cond.is_satisfied());
        let cond2 = ResumeCondition::OnCountReached { current: 10, target: 10 };
        assert!(cond2.is_satisfied());
        let cond3 = ResumeCondition::OnCountReached { current: 15, target: 10 };
        assert!(cond3.is_satisfied());
    }

    #[test]
    fn test_resume_after_duration() {
        let cond = ResumeCondition::AfterDuration(Duration::from_millis(100));
        let start = Instant::now();
        assert!(!cond.is_duration_elapsed(start));
        // After sleeping, it should be elapsed
        // (We can't reliably sleep in tests, so just test the logic)
    }

    #[test]
    fn test_resume_on_dependency() {
        let cond = ResumeCondition::OnDependencyReady("database".into());
        assert!(!cond.is_satisfied()); // Needs external resolution
    }

    // --- PauseEffect tests ---

    #[test]
    fn test_effect_refreshed() {
        let effect = PauseEffect::Refreshed {
            quality_boost: 1.2,
            speed_boost: 1.1,
        };
        assert!(effect.is_positive());
        assert!((effect.performance_score() - 1.15).abs() < 0.01);
    }

    #[test]
    fn test_effect_neutral() {
        let effect = PauseEffect::Neutral;
        assert!(!effect.is_positive());
        assert_eq!(effect.performance_score(), 1.0);
    }

    #[test]
    fn test_effect_momentum_loss() {
        let effect = PauseEffect::MomentumLoss {
            warmup_duration: Duration::from_secs(30),
        };
        assert!(!effect.is_positive());
        assert!(effect.performance_score() < 1.0);
    }

    #[test]
    fn test_effect_stale() {
        let effect = PauseEffect::Stale { staleness: 0.8 };
        assert!(!effect.is_positive());
        assert!((effect.performance_score() - 0.2).abs() < 0.01);
    }

    // --- Fermata tests ---

    #[test]
    fn test_fermata_creation() {
        let f = Fermata::new("pause-1", PauseReason::WaitingForInput, Duration::from_secs(5));
        assert_eq!(f.id(), "pause-1");
        assert_eq!(f.reason(), PauseReason::WaitingForInput);
        assert!(!f.is_active());
        assert_eq!(f.min_hold(), Duration::from_secs(5));
    }

    #[test]
    fn test_fermata_brief() {
        let f = Fermata::brief("quick", PauseReason::StrategicTiming);
        assert_eq!(f.min_hold(), Duration::from_millis(100));
    }

    #[test]
    fn test_fermata_long() {
        let f = Fermata::long("extended", PauseReason::Processing);
        assert_eq!(f.min_hold(), Duration::from_secs(30));
    }

    #[test]
    fn test_fermata_start_and_resume() {
        let mut f = Fermata::new("p1", PauseReason::WaitingForInput, Duration::from_secs(5));
        f.start();
        assert!(f.is_active());
        assert!(f.elapsed().is_some());
        // With AfterDuration(5s) and only ~0s elapsed, should not resume yet
        // unless the condition is immediately met
    }

    #[test]
    fn test_fermata_force_resume() {
        let mut f = Fermata::new("p1", PauseReason::Processing, Duration::from_secs(10))
            .with_effect(PauseEffect::Refreshed { quality_boost: 1.3, speed_boost: 1.0 });
        f.start();
        let effect = f.force_resume();
        assert!(!f.is_active());
        assert_eq!(effect, PauseEffect::Refreshed { quality_boost: 1.3, speed_boost: 1.0 });
    }

    #[test]
    fn test_fermata_with_max_hold() {
        let f = Fermata::new("p1", PauseReason::StrategicTiming, Duration::from_secs(1))
            .with_max_hold(Duration::from_secs(60));
        assert_eq!(f.max_hold(), Some(Duration::from_secs(60)));
    }

    #[test]
    fn test_fermata_with_resume_condition() {
        let f = Fermata::new("p1", PauseReason::WaitingForDependency, Duration::from_secs(0))
            .with_resume_condition(ResumeCondition::OnDependencyReady("api".into()));
        match f.resume_condition() {
            ResumeCondition::OnDependencyReady(dep) => assert_eq!(dep, "api"),
            _ => panic!("Expected OnDependencyReady"),
        }
    }

    #[test]
    fn test_fermata_with_priority() {
        let f = Fermata::new("p1", PauseReason::SystemHealth, Duration::from_secs(1))
            .with_priority(10);
        assert_eq!(f.priority(), 10);
    }

    #[test]
    fn test_fermata_with_metadata() {
        let f = Fermata::new("p1", PauseReason::Observation, Duration::from_secs(1))
            .with_metadata("source", "sensor-A")
            .with_metadata("threshold", "0.95");
        assert_eq!(f.metadata().get("source"), Some(&"sensor-A".to_string()));
        assert_eq!(f.metadata().get("threshold"), Some(&"0.95".to_string()));
    }

    #[test]
    fn test_fermata_immediate_resume() {
        let mut f = Fermata::new("p1", PauseReason::Processing, Duration::from_secs(0))
            .with_resume_condition(ResumeCondition::Immediate);
        f.start();
        assert!(f.should_resume());
    }

    #[test]
    fn test_fermata_manual_never_resumes() {
        let mut f = Fermata::new("p1", PauseReason::UserRequested, Duration::from_secs(0))
            .with_resume_condition(ResumeCondition::Manual);
        f.start();
        assert!(!f.should_resume());
    }

    #[test]
    fn test_fermata_not_started_resumes() {
        let f = Fermata::new("p1", PauseReason::Processing, Duration::from_secs(100));
        assert!(f.should_resume()); // not active = can resume
    }

    // --- PauseDetector tests ---

    #[test]
    fn test_pause_detector_creation() {
        let pd = PauseDetector::new(Duration::from_secs(5));
        assert_eq!(pd.event_count(), 0);
    }

    #[test]
    fn test_pause_detector_empty_is_good() {
        let pd = PauseDetector::new(Duration::from_secs(5));
        assert!(pd.is_good_pause_point());
    }

    #[test]
    fn test_pause_detector_records_events() {
        let mut pd = PauseDetector::new(Duration::from_secs(5));
        pd.record_event("task-complete");
        pd.record_event("task-complete");
        assert_eq!(pd.event_count(), 2);
    }

    #[test]
    fn test_pause_detector_boundary_event() {
        let mut pd = PauseDetector::new(Duration::from_secs(60));
        pd.add_boundary_event("phase-end");
        pd.record_event("phase-end");
        assert!(pd.is_good_pause_point());
    }

    #[test]
    fn test_pause_detector_time_since_last() {
        let mut pd = PauseDetector::new(Duration::from_secs(1));
        pd.record_event("tick");
        let elapsed = pd.time_since_last_event();
        assert!(elapsed.is_some());
        assert!(elapsed.unwrap() < Duration::from_secs(1));
    }

    #[test]
    fn test_pause_detector_clear() {
        let mut pd = PauseDetector::new(Duration::from_secs(5));
        pd.record_event("a");
        pd.record_event("b");
        pd.clear();
        assert_eq!(pd.event_count(), 0);
    }

    // --- BreathMark tests ---

    #[test]
    fn test_breath_mark_creation() {
        let bm = BreathMark::new(Duration::from_secs(30), Duration::from_millis(500));
        assert_eq!(bm.interval(), Duration::from_secs(30));
        assert_eq!(bm.breath_duration(), Duration::from_millis(500));
        assert!(bm.is_enabled());
    }

    #[test]
    fn test_breath_mark_needs_breath_initially() {
        let bm = BreathMark::new(Duration::from_secs(60), Duration::from_secs(1));
        assert!(bm.needs_breath()); // No breath taken yet
    }

    #[test]
    fn test_breath_mark_after_taking_breath() {
        let mut bm = BreathMark::new(Duration::from_secs(600), Duration::from_secs(1));
        bm.take_breath();
        assert!(!bm.needs_breath()); // Just took one
    }

    #[test]
    fn test_breath_mark_ops_limit() {
        let mut bm = BreathMark::new(Duration::from_secs(6000), Duration::from_secs(1))
            .with_max_ops(5);
        bm.take_breath(); // Reset
        for _ in 0..5 {
            bm.record_op();
        }
        assert!(bm.needs_breath());
        assert_eq!(bm.ops_since_breath(), 5);
    }

    #[test]
    fn test_breath_mark_disabled() {
        let mut bm = BreathMark::default_breath();
        bm.set_enabled(false);
        assert!(!bm.needs_breath());
        assert!(!bm.is_enabled());
    }

    #[test]
    fn test_breath_mark_default() {
        let bm = BreathMark::default_breath();
        assert_eq!(bm.interval(), Duration::from_secs(60));
        assert_eq!(bm.breath_duration(), Duration::from_secs(1));
    }

    #[test]
    fn test_breath_mark_ops_reset() {
        let mut bm = BreathMark::new(Duration::from_secs(6000), Duration::from_secs(1))
            .with_max_ops(3);
        bm.take_breath();
        bm.record_op();
        bm.record_op();
        assert_eq!(bm.ops_since_breath(), 2);
        bm.take_breath();
        assert_eq!(bm.ops_since_breath(), 0);
    }
}
