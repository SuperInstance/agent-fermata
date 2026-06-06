# agent-fermata

> *Fermata* (𝄐) — a musical symbol meaning "hold this note as long as it feels
> right." Not a delay, not a mistake — a deliberate, expressive pause. Sometimes
> the most important thing an agent can do is to wait.

**Strategic pauses for agent systems.**

## Overview

Agent systems are optimized for throughput and latency — do more, do it faster,
never stop. But real expertise includes knowing when to pause. A musician holds
a fermata to let a moment breathe. An agent should do the same: wait for the
right input, let processing complete, observe before acting, or simply rest
for system health.

`agent-fermata` provides the building blocks for strategic agent pauses:

- **Fermata** — A configurable pause point with minimum and maximum hold
  durations, resume conditions, and effects on subsequent performance. Start
  a fermata, and the agent holds until it feels right to continue.

- **PauseReason** — Categorize why the agent is pausing: waiting for input,
  processing internally, strategic timing, system health, waiting for a
  dependency, observation, or user request.

- **PauseDetector** — Identify natural pause points in a stream of events.
  The detector watches for lulls, boundaries, and transitions where a pause
  would be natural and beneficial.

- **ResumeCondition** — Define what triggers continuation: a fixed duration,
  an external condition, input arrival, dependency resolution, a counter
  threshold, manual intervention, or immediate resumption.

- **PauseEffect** — Model how pauses affect subsequent performance: refreshed
  (quality/speed boost), neutral, momentum loss (needs warmup), or stale
  (context degraded). Compute performance scores to decide if a pause was
  worth it.

- **BreathMark** — Mandatory periodic pauses for system health, like a
  musician's breath mark. Ensures the agent pauses at regular intervals or
  after a certain number of operations, regardless of workload.

## When to Use This

- **Rate limiting** — Your agent needs to pause between API calls, processing
  batches, or sending messages.
- **Waiting for input** — The agent should pause gracefully while waiting for
  user input, external APIs, or dependent services.
- **Strategic timing** — Sometimes delaying an action by a few seconds produces
  better results than acting immediately.
- **System health** — Agents need periodic cooldown periods to prevent resource
  exhaustion, memory leaks, or quota violations.
- **Observation** — Before acting, the agent pauses to observe the current
  state, gather context, or monitor for changes.

## Quick Start

```rust
use agent_fermata::{Fermata, PauseReason, ResumeCondition, PauseEffect, BreathMark};
use std::time::Duration;

// Create a strategic pause
let mut fermata = Fermata::new("wait-for-api", PauseReason::WaitingForInput, Duration::from_secs(5))
    .with_max_hold(Duration::from_secs(30))
    .with_effect(PauseEffect::Refreshed {
        quality_boost: 1.2,
        speed_boost: 1.0,
    });

fermata.start();
// ... later:
if fermata.should_resume() {
    let effect = fermata.force_resume();
}

// Set up mandatory breathing
let mut breath = BreathMark::new(Duration::from_secs(60), Duration::from_secs(1))
    .with_max_ops(1000);

for _ in 0..500 {
    breath.record_op();
    if breath.needs_breath() {
        breath.take_breath();
        // ... pause for breath.breath_duration()
    }
}
```

## Core Concepts

### The Fermata Philosophy

In music, a fermata isn't wasted time — it's expressive time. The note is held
longer than written because the moment demands it. Similarly, agent pauses aren't
bugs or inefficiencies — they're strategic decisions that improve overall
performance and reliability.

### Pause Effects

Not all pauses are equal. A short pause can refresh the agent (better quality,
faster subsequent work). A pause that's too long can cause momentum loss or
context staleness. `PauseEffect` models this spectrum so you can reason about
the cost and benefit of pausing.

### Breath Marks

Every system needs to breathe. `BreathMark` ensures the agent takes periodic
health pauses — either on a time interval or after a number of operations —
even when the agent is busy. Like a musician who must breathe regardless of
the tempo, an agent must pause regardless of its workload.

## License

MIT
