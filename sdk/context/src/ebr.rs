use crossbeam_epoch::{Atomic, Guard, Owned};
use std::{
    marker::PhantomData,
    sync::{atomic::Ordering, Arc},
};

/// Application-wide state management using epoch-based memory reclamation.
/// Before using it, make sure operations on `T` is read-heavy. [`Context`]
/// helps reduce the read overhead of Mutex when multiple threads access the
/// same object that is written rarely (where `T` is more than 90% read and less
/// than 10% write).
pub struct SharedContext<T> {
    ptr: Arc<Atomic<T>>,
}

unsafe impl<T> Send for SharedContext<T> {}

unsafe impl<T> Sync for SharedContext<T> {}

impl<T> Clone for SharedContext<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr.clone(),
        }
    }
}

impl<T> From<T> for SharedContext<T> {
    fn from(value: T) -> Self {
        Self {
            ptr: Arc::new(Atomic::new(value)),
        }
    }
}

impl<T> SharedContext<T> {
    fn as_ptr(&self) -> Arc<Atomic<T>> {
        self.ptr.clone()
    }

    /// Thread-safe getter for the current context.
    ///
    /// # Examples
    ///
    /// ```
    /// let context = SharedContext::from(u8::from(1));
    ///
    /// let current = context.load();
    /// let count = local.as_ref();
    /// println!("{:?}", user); // Prints '1'
    /// ```
    pub fn load(&self) -> Context<T> {
        Context::new(self.clone())
    }

    /// Thread-safe setter to save the new context `T`. After calling the
    /// function, the previous context becomes unreachable.
    ///
    /// # Examples
    ///
    /// ```
    /// let context = SharedContext::from(u8::from(1));
    ///
    /// let current = context.load();
    /// let count = current.as_ref();
    /// println!("{:?}", count); // Prints '1'
    ///
    /// let mut new_count = count.clone();
    /// new_count += 1;
    /// context.store(new_count);
    ///
    /// let current = context.load();
    /// let count = current.as_ref();
    /// println!("{:?}", count); // Prints '2'
    /// ```
    pub fn store(&self, context: T) {
        let guard = crossbeam_epoch::pin();
        let previous_context = self.ptr.swap(Owned::new(context), Ordering::SeqCst, &guard);

        unsafe { guard.defer_destroy(previous_context) }
        guard.flush();
    }

    /// Setter for the new context where there is a causal relationship between
    /// state changes. Like [`SharedContext::store()`], the previous state
    /// becomes unreachable after calling the function.
    ///
    /// The difference between [`SharedContext::update()`] and
    /// [`SharedContext::store()`] is that the former may fail when there
    /// are two or more threads updating the current context. Use this function
    /// if the new state depends on the current state.
    ///
    /// # CAVEAT
    /// Although it is okay to have more than one thread updating the current
    /// context, the practice is not advised considering the purpose of using
    /// [`SharedContext`],
    ///
    /// # Examples
    ///
    /// ```
    /// let context = SharedContext::from(u8::from(1));
    ///
    /// let current = context.load();
    /// let count = current.as_ref();
    /// println!("{:?}", count); // Prints '1'
    ///
    /// let mut new_count = count.clone();
    /// new_count += 1;
    /// context.update(new_count).unwrap();
    ///
    /// let current = context.load();
    /// let count = current.as_ref();
    /// println!("{:?}", count);
    /// ```
    pub fn update(&self, context: T) -> Result<(), ContextError> {
        let guard = crossbeam_epoch::pin();
        let current_context = self.ptr.load(Ordering::SeqCst, &guard);
        self.ptr
            .compare_exchange(
                current_context,
                Owned::new(context),
                Ordering::SeqCst,
                Ordering::SeqCst,
                &guard,
            )
            .map_err(|_| ContextError::Update)?;

        Ok(())
    }
}

pub struct Context<T> {
    shared_context: SharedContext<T>,
    guard: Guard,
    _not_send: PhantomData<NotSend>,
}

impl<T> AsRef<T> for Context<T> {
    fn as_ref(&self) -> &T {
        unsafe {
            self.shared_context
                .as_ptr()
                .load(Ordering::SeqCst, &self.guard)
                .as_ref()
                .unwrap()
        }
    }
}

impl<T> Context<T> {
    fn new(context: SharedContext<T>) -> Self {
        Self {
            shared_context: context,
            guard: crossbeam_epoch::pin(),
            _not_send: PhantomData,
        }
    }
}

#[allow(unused)]
struct NotSend(*const ());

pub enum ContextError {
    Update,
}

impl std::fmt::Debug for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // If you are seeing this error too often, check if there's more than one thread/task
            // updating the context concurrently.
            Self::Update => write!(f, "Context changed while getting updated"),
        }
    }
}

impl std::fmt::Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ContextError {}
