use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::{watch, Mutex, Notify};
use tokio::task::JoinHandle;

/// Represents the activation/wake state of the system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemState {
    /// System is sleeping, only listening for wake word
    Asleep,
    /// System is awake, processing commands and dictation
    Awake,
}

/// Reason for a state transition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateTransition {
    /// Transition triggered by wake word detection
    WakeWord,
    /// Transition triggered by sleep command
    SleepCommand,
    /// Transition triggered by inactivity timeout
    InactivityTimeout,
}

/// Manages the activation state and auto-sleep timer
pub struct ActivationManager {
    /// Watch channel sender for broadcasting state changes
    state_tx: watch::Sender<(SystemState, StateTransition)>,
    /// Shared state for the background timer task
    inner: Arc<ManagerInner>,
    /// Handle to the background timer task
    _timer_task: JoinHandle<()>,
}

/// Internal state managed by the manager
struct ManagerInner {
    /// Current activation state
    state: Mutex<SystemState>,
    /// Current transition reason
    transition: Mutex<StateTransition>,
    /// Timeout duration in seconds (stored atomically to allow dynamic updates without locking)
    timeout_secs: AtomicU64,
    /// Notification channel for activity events
    activity: Notify,
    /// Notification channel for timeout changes
    timeout_changed: Notify,
}

impl ActivationManager {
    /// Creates a new ActivationManager
    ///
    /// # Arguments
    /// * `timeout_secs` - Time in seconds before auto-sleep after last activity
    ///
    /// # Returns
    /// A new `ActivationManager` in the `Asleep` state with the timer task spawned
    pub fn new(timeout_secs: u64) -> Self {
        let initial_state = SystemState::Asleep;
        let initial_transition = StateTransition::WakeWord;
        let (state_tx, _state_rx) = watch::channel((initial_state, initial_transition));

        let inner = Arc::new(ManagerInner {
            state: Mutex::new(initial_state),
            transition: Mutex::new(initial_transition),
            timeout_secs: AtomicU64::new(timeout_secs),
            activity: Notify::new(),
            timeout_changed: Notify::new(),
        });

        let timer_task = Self::spawn_timer_task(inner.clone(), state_tx.clone());

        tracing::info!("ActivationManager initialized with timeout: {}s", timeout_secs);

        Self {
            state_tx,
            inner,
            _timer_task: timer_task,
        }
    }

    /// Returns a receiver that can be used to subscribe to state changes
    pub fn subscribe(&self) -> watch::Receiver<(SystemState, StateTransition)> {
        self.state_tx.subscribe()
    }

    /// Updates the auto-sleep timeout duration
    ///
    /// This immediately affects the running timer, restarting it with the new duration.
    /// Shorter timeouts begin enforcing immediately, and longer timeouts extend the
    /// current idle period.
    ///
    /// # Arguments
    /// * `timeout` - New timeout duration
    pub async fn set_timeout(&self, timeout: Duration) {
        let secs = timeout.as_secs();
        self.inner.timeout_secs.store(secs, Ordering::Release);
        tracing::debug!("Updated auto-sleep timeout to: {:?}", timeout);
        // Notify the timer task to restart with the new timeout
        self.inner.timeout_changed.notify_one();
    }

    /// Returns the current state
    pub fn current_state(&self) -> SystemState {
        let state = match self.inner.state.try_lock() {
            Ok(guard) => *guard,
            Err(_) => {
                self.state_tx.borrow().0
            }
        };
        state
    }

    /// Returns the reason for the current state
    pub fn current_transition(&self) -> StateTransition {
        self.state_tx.borrow().1
    }

    /// Wake the system via wake word detection
    ///
    /// Transitions from `Asleep` to `Awake` and logs the transition.
    pub async fn wake_via_wake_word(&self) {
        let mut state = self.inner.state.lock().await;
        if *state == SystemState::Asleep {
            *state = SystemState::Awake;
            drop(state);
            let mut transition = self.inner.transition.lock().await;
            *transition = StateTransition::WakeWord;
            drop(transition);
            let _ = self.state_tx.send((SystemState::Awake, StateTransition::WakeWord));
            tracing::info!("State transition: Asleep -> Awake (via wake word)");
            self.inner.activity.notify_one();
        }
    }

    /// Sleep the system via sleep command
    ///
    /// Transitions from `Awake` to `Asleep` and logs the transition.
    pub async fn sleep_via_command(&self) {
        let mut state = self.inner.state.lock().await;
        if *state == SystemState::Awake {
            *state = SystemState::Asleep;
            drop(state);
            let mut transition = self.inner.transition.lock().await;
            *transition = StateTransition::SleepCommand;
            drop(transition);
            let _ = self.state_tx.send((SystemState::Asleep, StateTransition::SleepCommand));
            tracing::info!("State transition: Awake -> Asleep (via sleep command)");
        }
    }

    /// Signal command activity while awake
    ///
    /// Resets the inactivity timer if the system is currently awake.
    pub async fn on_command_activity(&self) {
        let state = self.inner.state.lock().await;
        if *state == SystemState::Awake {
            drop(state);
            tracing::debug!("Command activity detected, resetting timer");
            self.inner.activity.notify_one();
        }
    }

    /// Signal dictation activity while awake
    ///
    /// Resets the inactivity timer if the system is currently awake.
    pub async fn on_dictation_activity(&self) {
        let state = self.inner.state.lock().await;
        if *state == SystemState::Awake {
            drop(state);
            tracing::debug!("Dictation activity detected, resetting timer");
            self.inner.activity.notify_one();
        }
    }

    /// Spawns the background timer task that monitors inactivity
    fn spawn_timer_task(
        inner: Arc<ManagerInner>,
        state_tx: watch::Sender<(SystemState, StateTransition)>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                loop {
                    let state = inner.state.lock().await;
                    if *state == SystemState::Awake {
                        drop(state);
                        break;
                    }
                    drop(state);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }

                tracing::debug!("Inactivity timer started");

                let timeout_secs = inner.timeout_secs.load(Ordering::Acquire);
                let sleep_future = tokio::time::sleep(Duration::from_secs(timeout_secs));
                tokio::pin!(sleep_future);

                loop {
                    tokio::select! {
                        _ = &mut sleep_future => {
                            let mut state = inner.state.lock().await;
                            if *state == SystemState::Awake {
                                *state = SystemState::Asleep;
                                drop(state);
                                let mut transition = inner.transition.lock().await;
                                *transition = StateTransition::InactivityTimeout;
                                drop(transition);
                                let _ = state_tx.send((SystemState::Asleep, StateTransition::InactivityTimeout));
                                tracing::info!("State transition: Awake -> Asleep (via inactivity timeout)");
                            }
                            break;
                        }
                        _ = inner.activity.notified() => {
                            tracing::debug!("Activity detected, resetting inactivity timer");
                            let state = inner.state.lock().await;
                            if *state == SystemState::Asleep {
                                drop(state);
                                break;
                            }
                            drop(state);
                            let timeout_secs = inner.timeout_secs.load(Ordering::Acquire);
                            sleep_future.set(tokio::time::sleep(Duration::from_secs(timeout_secs)));
                        }
                        _ = inner.timeout_changed.notified() => {
                            tracing::debug!("Timeout changed, restarting inactivity timer");
                            let state = inner.state.lock().await;
                            if *state == SystemState::Asleep {
                                drop(state);
                                break;
                            }
                            drop(state);
                            let timeout_secs = inner.timeout_secs.load(Ordering::Acquire);
                            sleep_future.set(tokio::time::sleep(Duration::from_secs(timeout_secs)));
                        }
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to ensure background timer task processes
    /// Uses the watch channel to detect state changes reliably
    async fn wait_for_state(manager: &ActivationManager, expected_state: SystemState) {
        if manager.current_state() == expected_state {
            return;
        }

        let mut subscriber = manager.subscribe();

        for _ in 0..100 {
            if manager.current_state() == expected_state {
                return;
            }
            tokio::task::yield_now().await;
        }

        for _ in 0..10 {
            if subscriber.changed().await.is_ok() {
                let (state, _) = *subscriber.borrow();
                if state == expected_state {
                    return;
                }
            } else {
                break;
            }
        }

        assert_eq!(manager.current_state(), expected_state, "Failed to reach expected state");
    }

    #[tokio::test]
    async fn test_initial_state_is_asleep() {
        let manager = ActivationManager::new(300);
        assert_eq!(manager.current_state(), SystemState::Asleep);
    }

    #[tokio::test]
    async fn test_wake_via_wake_word() {
        let manager = ActivationManager::new(300);
        assert_eq!(manager.current_state(), SystemState::Asleep);

        manager.wake_via_wake_word().await;
        assert_eq!(manager.current_state(), SystemState::Awake);
    }

    #[tokio::test]
    async fn test_sleep_via_command() {
        let manager = ActivationManager::new(300);
        manager.wake_via_wake_word().await;
        assert_eq!(manager.current_state(), SystemState::Awake);

        manager.sleep_via_command().await;
        assert_eq!(manager.current_state(), SystemState::Asleep);
    }

    #[tokio::test]
    async fn test_wake_is_idempotent() {
        let manager = ActivationManager::new(300);
        manager.wake_via_wake_word().await;
        manager.wake_via_wake_word().await;
        assert_eq!(manager.current_state(), SystemState::Awake);
    }

    #[tokio::test]
    async fn test_sleep_is_idempotent() {
        let manager = ActivationManager::new(300);
        manager.sleep_via_command().await;
        manager.sleep_via_command().await;
        assert_eq!(manager.current_state(), SystemState::Asleep);
    }

    #[tokio::test]
    async fn test_activity_resets_timer() {
        tokio::time::pause();
        let manager = ActivationManager::new(1);
        manager.wake_via_wake_word().await;

        tokio::time::advance(Duration::from_millis(500)).await;
        wait_for_state(&manager, SystemState::Awake).await;

        manager.on_command_activity().await;

        tokio::time::advance(Duration::from_millis(500)).await;
        wait_for_state(&manager, SystemState::Awake).await;

        tokio::time::advance(Duration::from_millis(600)).await;
        wait_for_state(&manager, SystemState::Asleep).await;
    }

    #[tokio::test]
    async fn test_auto_sleep_on_timeout() {
        tokio::time::pause();
        let manager = ActivationManager::new(1);
        manager.wake_via_wake_word().await;

        tokio::time::advance(Duration::from_millis(1500)).await;
        wait_for_state(&manager, SystemState::Asleep).await;
    }

    #[tokio::test]
    async fn test_command_activity_while_awake() {
        tokio::time::pause();
        let manager = ActivationManager::new(2);
        manager.wake_via_wake_word().await;

        manager.on_command_activity().await;
        tokio::time::advance(Duration::from_millis(600)).await;
        tokio::task::yield_now().await;
        assert_eq!(manager.current_state(), SystemState::Awake);
    }

    #[tokio::test]
    async fn test_dictation_activity_while_awake() {
        tokio::time::pause();
        let manager = ActivationManager::new(2);
        manager.wake_via_wake_word().await;

        manager.on_dictation_activity().await;
        tokio::time::advance(Duration::from_millis(600)).await;
        tokio::task::yield_now().await;
        assert_eq!(manager.current_state(), SystemState::Awake);
    }

    #[tokio::test]
    async fn test_activity_while_asleep_does_nothing() {
        tokio::time::pause();
        let manager = ActivationManager::new(300);
        assert_eq!(manager.current_state(), SystemState::Asleep);

        manager.on_command_activity().await;
        tokio::time::advance(Duration::from_millis(100)).await;
        tokio::task::yield_now().await;
        assert_eq!(manager.current_state(), SystemState::Asleep);

        manager.on_dictation_activity().await;
        tokio::time::advance(Duration::from_millis(100)).await;
        tokio::task::yield_now().await;
        assert_eq!(manager.current_state(), SystemState::Asleep);
    }

    #[tokio::test]
    async fn test_subscriber_receives_state_updates() {
        let manager = ActivationManager::new(300);
        let mut subscriber = manager.subscribe();

        let (state, _) = *subscriber.borrow();
        assert_eq!(state, SystemState::Asleep);

        manager.wake_via_wake_word().await;
        assert!(subscriber.changed().await.is_ok());
        let (state, reason) = *subscriber.borrow();
        assert_eq!(state, SystemState::Awake);
        assert_eq!(reason, StateTransition::WakeWord);

        manager.sleep_via_command().await;
        assert!(subscriber.changed().await.is_ok());
        let (state, reason) = *subscriber.borrow();
        assert_eq!(state, SystemState::Asleep);
        assert_eq!(reason, StateTransition::SleepCommand);
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let manager = ActivationManager::new(300);
        let mut sub1 = manager.subscribe();
        let mut sub2 = manager.subscribe();

        let (state, _) = *sub1.borrow();
        assert_eq!(state, SystemState::Asleep);
        let (state, _) = *sub2.borrow();
        assert_eq!(state, SystemState::Asleep);

        manager.wake_via_wake_word().await;
        assert!(sub1.changed().await.is_ok());
        assert!(sub2.changed().await.is_ok());

        let (state, _) = *sub1.borrow();
        assert_eq!(state, SystemState::Awake);
        let (state, _) = *sub2.borrow();
        assert_eq!(state, SystemState::Awake);
    }

    #[tokio::test]
    async fn test_current_state_returns_latest() {
        let manager = ActivationManager::new(300);
        assert_eq!(manager.current_state(), SystemState::Asleep);

        manager.wake_via_wake_word().await;
        assert_eq!(manager.current_state(), SystemState::Awake);

        manager.sleep_via_command().await;
        assert_eq!(manager.current_state(), SystemState::Asleep);
    }

    #[tokio::test]
    async fn test_sleep_command_resets_timer() {
        tokio::time::pause();
        let manager = ActivationManager::new(3);
        manager.wake_via_wake_word().await;

        tokio::time::advance(Duration::from_millis(500)).await;
        tokio::task::yield_now().await;
        manager.sleep_via_command().await;

        tokio::time::advance(Duration::from_millis(2000)).await;
        tokio::task::yield_now().await;
        assert_eq!(manager.current_state(), SystemState::Asleep);
    }

    #[tokio::test]
    async fn test_multiple_activities_reset_timer() {
        tokio::time::pause();
        let manager = ActivationManager::new(2);
        manager.wake_via_wake_word().await;

        for _ in 0..3 {
            tokio::time::advance(Duration::from_millis(600)).await;
            tokio::task::yield_now().await;
            manager.on_command_activity().await;
        }

        wait_for_state(&manager, SystemState::Awake).await;

        tokio::time::advance(Duration::from_millis(2100)).await;
        wait_for_state(&manager, SystemState::Asleep).await;
    }

    #[tokio::test]
    async fn test_set_timeout_updates_timer() {
        tokio::time::pause();
        let manager = ActivationManager::new(10);
        manager.wake_via_wake_word().await;

        tokio::time::advance(Duration::from_secs(2)).await;
        wait_for_state(&manager, SystemState::Awake).await;

        manager.set_timeout(Duration::from_secs(1)).await;
        tracing::info!("Timeout updated to 1 second");

        manager.on_command_activity().await;

        tokio::time::advance(Duration::from_millis(1200)).await;
        wait_for_state(&manager, SystemState::Asleep).await;
    }

    #[tokio::test]
    async fn test_set_timeout_extends_timeout() {
        tokio::time::pause();
        let manager = ActivationManager::new(1);
        manager.wake_via_wake_word().await;

        manager.set_timeout(Duration::from_secs(5)).await;

        tokio::time::advance(Duration::from_secs(2)).await;
        wait_for_state(&manager, SystemState::Awake).await;

        tokio::time::advance(Duration::from_secs(2)).await;
        wait_for_state(&manager, SystemState::Awake).await;

        tokio::time::advance(Duration::from_millis(1500)).await;
        wait_for_state(&manager, SystemState::Asleep).await;
    }
}
