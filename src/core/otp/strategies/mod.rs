/// Restart strategies and policies for supervisors
/// Implements various restart patterns from OTP
use super::*;
use std::time::{Duration, Instant};
use tracing::{debug, error, info};

/// Implementation of restart strategies
pub struct RestartPolicyExecutor {
    strategy: RestartStrategy,
    max_restarts: u32,
    time_window: Duration,
}

impl RestartPolicyExecutor {
    pub fn new(strategy: RestartStrategy) -> Self {
        let (max_restarts, time_window) = match &strategy {
            RestartStrategy::OneForOne {
                max_restarts,
                time_window,
            }
            | RestartStrategy::OneForAll {
                max_restarts,
                time_window,
            }
            | RestartStrategy::RestForOne {
                max_restarts,
                time_window,
            }
            | RestartStrategy::SimpleOneForOne {
                max_restarts,
                time_window,
            } => (*max_restarts, *time_window),
        };

        Self {
            strategy,
            max_restarts,
            time_window,
        }
    }

    /// Check if a child should be restarted based on policy and history
    pub fn should_restart_child(&self, child_history: &ChildRestartHistory) -> RestartDecision {
        match &self.strategy {
            RestartStrategy::OneForOne { .. } => self.check_simple_limits(child_history),
            RestartStrategy::OneForAll { .. } => {
                // For one-for-all, if any child restarts, all children restart
                RestartDecision::RestartAll {
                    delay: Duration::from_millis(100),
                }
            }
            RestartStrategy::RestForOne { .. } => {
                // For rest-for-one, the failed child and all subsequent children restart
                RestartDecision::RestartFromIndex {
                    index: child_history.child_index,
                    delay: Duration::from_millis(50),
                }
            }
            RestartStrategy::SimpleOneForOne { .. } => {
                // For simple one-for-one, always restart transient workers
                if child_history.restart_policy == RestartPolicy::Permanent
                    || child_history.restart_policy == RestartPolicy::Transient
                {
                    self.check_simple_limits(child_history)
                } else {
                    RestartDecision::NoRestart
                }
            }
        }
    }

    fn check_simple_limits(&self, child_history: &ChildRestartHistory) -> RestartDecision {
        // Count restarts within time window
        let recent_restarts = child_history
            .restart_times
            .iter()
            .filter(|&&time| time.elapsed() <= self.time_window)
            .count();

        if recent_restarts >= self.max_restarts as usize {
            error!(
                "Restart limit exceeded for child: {} restarts in {:?}",
                recent_restarts, self.time_window
            );
            RestartDecision::NoRestart
        } else {
            // Calculate exponential backoff delay
            let delay = self.calculate_backoff_delay(recent_restarts);
            RestartDecision::Restart { delay }
        }
    }

    fn calculate_backoff_delay(&self, restart_count: usize) -> Duration {
        // Exponential backoff: 100ms, 200ms, 400ms, 800ms, max 5s
        let base_delay = Duration::from_millis(100);
        let max_delay = Duration::from_secs(5);
        let exponential_delay = base_delay * 2_u32.pow(restart_count as u32);
        exponential_delay.min(max_delay)
    }

    pub fn strategy(&self) -> &RestartStrategy {
        &self.strategy
    }
}

/// History of child restarts for making restart decisions
#[derive(Debug, Clone)]
pub struct ChildRestartHistory {
    pub child_id: ActorId,
    pub child_index: usize,
    pub restart_policy: RestartPolicy,
    pub restart_times: Vec<Instant>,
    pub last_restart: Option<Instant>,
    pub total_restarts: u32,
}

impl ChildRestartHistory {
    pub fn new(child_id: ActorId, child_index: usize, restart_policy: RestartPolicy) -> Self {
        Self {
            child_id,
            child_index,
            restart_policy,
            restart_times: Vec::new(),
            last_restart: None,
            total_restarts: 0,
        }
    }

    pub fn record_restart(&mut self) {
        let now = Instant::now();
        self.restart_times.push(now);
        self.last_restart = Some(now);
        self.total_restarts += 1;

        // Cleanup old restart times outside window
        self.cleanup_old_restarts(Duration::from_secs(3600)); // Keep 1 hour of history
    }

    pub fn cleanup_old_restarts(&mut self, older_than: Duration) {
        let cutoff = Instant::now() - older_than;
        self.restart_times.retain(|&time| time >= cutoff);
    }

    pub fn restarts_in_window(&self, window: Duration) -> usize {
        let cutoff = Instant::now() - window;
        self.restart_times
            .iter()
            .filter(|&&time| time >= cutoff)
            .count()
    }
}

/// Decision made by restart policy
#[derive(Debug, Clone)]
pub enum RestartDecision {
    /// Restart the specific child with delay
    Restart { delay: Duration },
    /// Restart all children with delay
    RestartAll { delay: Duration },
    /// Restart from this child and all subsequent children
    RestartFromIndex { index: usize, delay: Duration },
    /// Do not restart the child
    NoRestart,
}

/// Advanced restart strategies with dynamic adjustment
#[derive(Debug, Clone)]
pub enum AdaptiveRestartStrategy {
    /// Standard fixed strategy
    Fixed(RestartStrategy),
    /// Adapt based on system load
    LoadAdaptive {
        base_strategy: RestartStrategy,
        load_threshold: f32,
        aggressive_multiplier: f32,
    },
    /// Adapt based on failure patterns
    PatternAdaptive {
        base_strategy: RestartStrategy,
        failure_window: Duration,
        pattern_threshold: f32,
    },
}

/// Adaptive restart policy executor
pub struct AdaptiveRestartExecutor {
    strategy: AdaptiveRestartStrategy,
    base_executor: RestartPolicyExecutor,
    system_monitor: Option<SystemLoadMonitor>,
    pattern_analyzer: Option<FailurePatternAnalyzer>,
}

impl AdaptiveRestartExecutor {
    pub fn new(strategy: AdaptiveRestartStrategy) -> Self {
        let base_executor = match &strategy {
            AdaptiveRestartStrategy::Fixed(base)
            | AdaptiveRestartStrategy::LoadAdaptive {
                base_strategy: base,
                ..
            }
            | AdaptiveRestartStrategy::PatternAdaptive {
                base_strategy: base,
                ..
            } => RestartPolicyExecutor::new(base.clone()),
        };

        let (system_monitor, pattern_analyzer) = match &strategy {
            AdaptiveRestartStrategy::LoadAdaptive { .. } => (Some(SystemLoadMonitor::new()), None),
            AdaptiveRestartStrategy::PatternAdaptive { .. } => {
                (None, Some(FailurePatternAnalyzer::new()))
            }
            _ => (None, None),
        };

        Self {
            strategy,
            base_executor,
            system_monitor,
            pattern_analyzer,
        }
    }

    pub fn should_restart_child(&self, child_history: &ChildRestartHistory) -> RestartDecision {
        match &self.strategy {
            AdaptiveRestartStrategy::Fixed(_) => {
                self.base_executor.should_restart_child(child_history)
            }
            AdaptiveRestartStrategy::LoadAdaptive {
                load_threshold,
                aggressive_multiplier,
                ..
            } => self.adaptive_load_restart(child_history, *load_threshold, *aggressive_multiplier),
            AdaptiveRestartStrategy::PatternAdaptive {
                pattern_threshold, ..
            } => self.adaptive_pattern_restart(child_history, *pattern_threshold),
        }
    }

    fn adaptive_load_restart(
        &self,
        child_history: &ChildRestartHistory,
        load_threshold: f32,
        aggressive_multiplier: f32,
    ) -> RestartDecision {
        let base_decision = self.base_executor.should_restart_child(child_history);

        if let Some(monitor) = &self.system_monitor {
            let current_load = monitor.get_current_load().unwrap_or(0.0);

            if current_load > load_threshold {
                // System is under high load, be more conservative
                match base_decision {
                    RestartDecision::Restart { delay } => {
                        let adjusted_delay = Duration::from_millis(
                            (delay.as_millis() as f64 * aggressive_multiplier) as u64,
                        );
                        info!("Adaptive restart: high load detected, increasing delay from {:?} to {:?}", 
                            delay, adjusted_delay);
                        RestartDecision::Restart {
                            delay: adjusted_delay,
                        }
                    }
                    other => other,
                }
            } else {
                // Normal load, use base decision
                base_decision
            }
        } else {
            base_decision
        }
    }

    fn adaptive_pattern_restart(
        &self,
        child_history: &ChildRestartHistory,
        pattern_threshold: f32,
    ) -> RestartDecision {
        let base_decision = self.base_executor.should_restart_child(child_history);

        if let Some(analyzer) = &self.pattern_analyzer {
            let pattern_score = analyzer.analyze_failure_pattern(child_history);

            if pattern_score > pattern_threshold {
                // Detected problematic pattern, be more conservative
                warn!("Adaptive restart: problematic pattern detected (score: {:.2}), being conservative", pattern_score);
                match base_decision {
                    RestartDecision::Restart { delay } => {
                        // Significantly increase delay for pattern-based restarts
                        let conservative_delay = Duration::from_secs(5);
                        RestartDecision::Restart {
                            delay: conservative_delay,
                        }
                    }
                    RestartDecision::RestartFromIndex { index, delay } => {
                        RestartDecision::RestartFromIndex {
                            index,
                            delay: Duration::from_secs(5),
                        }
                    }
                    other => other,
                }
            } else {
                base_decision
            }
        } else {
            base_decision
        }
    }
}

/// Monitor system load for adaptive restarts
pub struct SystemLoadMonitor {
    cpu_threshold: f32,
    memory_threshold: f32,
}

impl SystemLoadMonitor {
    pub fn new() -> Self {
        Self {
            cpu_threshold: 80.0,
            memory_threshold: 80.0,
        }
    }

    pub fn get_current_load(&self) -> Option<f32> {
        // This would actually monitor system load
        // For now, return a simulated value
        Some(50.0) // 50% load
    }
}

/// Analyze failure patterns for adaptive restarts
pub struct FailurePatternAnalyzer {
    failure_window: Duration,
}

impl FailurePatternAnalyzer {
    pub fn new() -> Self {
        Self {
            failure_window: Duration::from_secs(300), // 5 minutes
        }
    }

    pub fn analyze_failure_pattern(&self, child_history: &ChildRestartHistory) -> f32 {
        let recent_restarts = child_history.restarts_in_window(self.failure_window);

        // Simple scoring: more restarts = higher score
        let restart_score = (recent_restarts as f32 / 10.0).min(1.0);

        // Check for rapid succession restarts
        let rapid_restart_score = if child_history.total_restarts > 3 {
            0.8
        } else {
            0.0
        };

        restart_score.max(rapid_restart_score)
    }
}

/// Utilities for restart strategies
pub mod utils {
    use super::*;

    pub fn create_restart_strategy(
        strategy_type: &str,
        max_restarts: u32,
        time_window_secs: u64,
    ) -> RestartStrategy {
        let time_window = Duration::from_secs(time_window_secs);

        match strategy_type {
            "one_for_one" => RestartStrategy::OneForOne {
                max_restarts,
                time_window,
            },
            "one_for_all" => RestartStrategy::OneForAll {
                max_restarts,
                time_window,
            },
            "rest_for_one" => RestartStrategy::RestForOne {
                max_restarts,
                time_window,
            },
            "simple_one_for_one" => RestartStrategy::SimpleOneForOne {
                max_restarts,
                time_window,
            },
            _ => RestartStrategy::OneForOne {
                max_restarts,
                time_window,
            },
        }
    }

    pub fn strategy_to_string(strategy: &RestartStrategy) -> &'static str {
        match strategy {
            RestartStrategy::OneForOne { .. } => "one_for_one",
            RestartStrategy::OneForAll { .. } => "one_for_all",
            RestartStrategy::RestForOne { .. } => "rest_for_one",
            RestartStrategy::SimpleOneForOne { .. } => "simple_one_for_one",
        }
    }

    pub fn validate_restart_strategy(strategy: &RestartStrategy) -> OtpResult<()> {
        match strategy {
            RestartStrategy::OneForOne {
                max_restarts,
                time_window,
            }
            | RestartStrategy::OneForAll {
                max_restarts,
                time_window,
            }
            | RestartStrategy::RestForOne {
                max_restarts,
                time_window,
            }
            | RestartStrategy::SimpleOneForOne {
                max_restarts,
                time_window,
            } => {
                if *max_restarts == 0 {
                    return Err(OtpError::SupervisorError {
                        reason: "Max restarts cannot be zero".to_string(),
                    });
                }

                if *time_window == Duration::ZERO {
                    return Err(OtpError::SupervisorError {
                        reason: "Time window cannot be zero".to_string(),
                    });
                }

                Ok(())
            }
        }
    }
}
