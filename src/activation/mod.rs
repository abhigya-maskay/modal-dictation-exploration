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
    timer_task: JoinHandle<()>,
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
            timer_task,
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

    /// Notify the system of ongoing activity to keep it awake
    ///
    /// This method should be called by dictation and command subsystems during
    /// active use to reset the inactivity timer and prevent auto-sleep.
    /// Unlike `wake_via_wake_word()`, this does not change the system state,
    /// it only extends the awake period by resetting the timer.
    ///
    /// # Example
    /// ```no_run
    /// // Call periodically during dictation processing
    /// activation_manager.notify_activity();
    /// ```
    pub fn notify_activity(&self) {
        tracing::debug!("Activity heartbeat received");
        self.inner.activity.notify_one();
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

impl Drop for ActivationManager {
    fn drop(&mut self) {
        self.timer_task.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initial_state_is_asleep() {
        let manager = ActivationManager::new(300);
        assert_eq!(manager.current_state(), SystemState::Asleep);
    }

    #[tokio::test]
    async fn test_wake_transition_via_wake_word() {
        let manager = ActivationManager::new(300);
        let mut rx = manager.subscribe();

        assert_eq!(manager.current_state(), SystemState::Asleep);

        manager.wake_via_wake_word().await;

        assert_eq!(manager.current_state(), SystemState::Awake);

        rx.changed().await.unwrap();
        let (state, transition) = *rx.borrow_and_update();
        assert_eq!(state, SystemState::Awake);
        assert_eq!(transition, StateTransition::WakeWord);

        manager.wake_via_wake_word().await;
        assert_eq!(manager.current_state(), SystemState::Awake);

        assert!(!rx.has_changed().unwrap());
    }

    #[tokio::test]
    async fn test_inactivity_auto_sleep() {
        tokio::time::pause();

        let timeout_secs = 2;
        let manager = ActivationManager::new(timeout_secs);
        let mut rx = manager.subscribe();

        manager.wake_via_wake_word().await;
        assert_eq!(manager.current_state(), SystemState::Awake);

        rx.changed().await.unwrap();

        tokio::time::sleep(Duration::from_millis(10)).await;

        tokio::time::advance(Duration::from_secs(timeout_secs + 1)).await;

        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(manager.current_state(), SystemState::Asleep);

        rx.changed().await.unwrap();
        let (state, transition) = *rx.borrow_and_update();
        assert_eq!(state, SystemState::Asleep);
        assert_eq!(transition, StateTransition::InactivityTimeout);
    }

    #[tokio::test]
    async fn test_runtime_timeout_changes() {
        tokio::time::pause();

        let initial_timeout = 10;
        let manager = ActivationManager::new(initial_timeout);
        let mut rx = manager.subscribe();

        manager.wake_via_wake_word().await;
        rx.changed().await.unwrap();

        tokio::time::advance(Duration::from_secs(5)).await;
        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(manager.current_state(), SystemState::Awake);

        manager.set_timeout(Duration::from_secs(2)).await;
        tokio::time::sleep(Duration::from_millis(10)).await;

        tokio::time::advance(Duration::from_secs(3)).await;
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(manager.current_state(), SystemState::Asleep);
        rx.changed().await.unwrap();

        manager.wake_via_wake_word().await;
        rx.changed().await.unwrap();

        manager.set_timeout(Duration::from_secs(10)).await;
        tokio::time::sleep(Duration::from_millis(10)).await;

        tokio::time::advance(Duration::from_secs(5)).await;
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(manager.current_state(), SystemState::Awake);
    }

    #[tokio::test]
    async fn test_notify_activity_heartbeat() {
        tokio::time::pause();

        let timeout_secs = 5;
        let manager = ActivationManager::new(timeout_secs);
        let mut rx = manager.subscribe();

        manager.wake_via_wake_word().await;
        rx.changed().await.unwrap();

        tokio::time::advance(Duration::from_secs(4)).await;
        tokio::time::sleep(Duration::from_millis(10)).await;

        manager.notify_activity();
        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(manager.current_state(), SystemState::Awake);

        tokio::time::advance(Duration::from_secs(4)).await;
        tokio::time::sleep(Duration::from_millis(10)).await;

        manager.notify_activity();
        tokio::time::sleep(Duration::from_millis(10)).await;

        assert_eq!(manager.current_state(), SystemState::Awake);

        tokio::time::advance(Duration::from_secs(6)).await;
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(manager.current_state(), SystemState::Asleep);
        rx.changed().await.unwrap();
        let (state, transition) = *rx.borrow_and_update();
        assert_eq!(state, SystemState::Asleep);
        assert_eq!(transition, StateTransition::InactivityTimeout);
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        tokio::time::pause();

        let timeout_secs = 2;
        let manager = ActivationManager::new(timeout_secs);

        let mut rx1 = manager.subscribe();
        let mut rx2 = manager.subscribe();
        let mut rx3 = manager.subscribe();

        manager.wake_via_wake_word().await;

        rx1.changed().await.unwrap();
        let (state1, transition1) = *rx1.borrow_and_update();
        assert_eq!(state1, SystemState::Awake);
        assert_eq!(transition1, StateTransition::WakeWord);

        rx2.changed().await.unwrap();
        let (state2, transition2) = *rx2.borrow_and_update();
        assert_eq!(state2, SystemState::Awake);
        assert_eq!(transition2, StateTransition::WakeWord);

        rx3.changed().await.unwrap();
        let (state3, transition3) = *rx3.borrow_and_update();
        assert_eq!(state3, SystemState::Awake);
        assert_eq!(transition3, StateTransition::WakeWord);

        tokio::time::advance(Duration::from_secs(timeout_secs + 1)).await;
        tokio::time::sleep(Duration::from_millis(100)).await;

        rx1.changed().await.unwrap();
        let (state1, transition1) = *rx1.borrow_and_update();
        assert_eq!(state1, SystemState::Asleep);
        assert_eq!(transition1, StateTransition::InactivityTimeout);

        rx2.changed().await.unwrap();
        let (state2, transition2) = *rx2.borrow_and_update();
        assert_eq!(state2, SystemState::Asleep);
        assert_eq!(transition2, StateTransition::InactivityTimeout);

        rx3.changed().await.unwrap();
        let (state3, transition3) = *rx3.borrow_and_update();
        assert_eq!(state3, SystemState::Asleep);
        assert_eq!(transition3, StateTransition::InactivityTimeout);
    }
}
