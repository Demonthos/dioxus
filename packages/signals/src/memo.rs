use crate::write::Writable;
use crate::{read::Readable, ReactiveContext, ReadableRef, Signal};
use crate::{CopyValue, ReadOnlySignal};
use std::{
    cell::RefCell,
    ops::Deref,
    panic::Location,
    sync::{atomic::AtomicBool, Arc},
};

use dioxus_core::prelude::*;
use futures_util::StreamExt;
use generational_box::UnsyncStorage;
struct UpdateInformation<T> {
    dirty: Arc<AtomicBool>,
    callback: RefCell<Box<dyn FnMut() -> T>>,
}

/// A value that is memoized. This is useful for caching the result of a computation.
pub struct Memo<T: 'static> {
    inner: Signal<T>,
    update: CopyValue<UpdateInformation<T>>,
}

impl<T> From<Memo<T>> for ReadOnlySignal<T>
where
    T: PartialEq,
{
    fn from(val: Memo<T>) -> Self {
        ReadOnlySignal::new(val.inner)
    }
}

impl<T: 'static> Memo<T> {
    /// Create a new memo
    #[track_caller]
    pub fn new(mut f: impl FnMut() -> T + 'static) -> Self
    where
        T: PartialEq,
    {
        let dirty = Arc::new(AtomicBool::new(true));
        let (tx, mut rx) = futures_channel::mpsc::unbounded();

        let callback = {
            let dirty = dirty.clone();
            move || {
                dirty.store(true, std::sync::atomic::Ordering::Relaxed);
                tx.unbounded_send(()).unwrap();
            }
        };
        let rc = ReactiveContext::new_with_callback(
            callback,
            current_scope_id().unwrap(),
            Location::caller(),
        );

        // Create a new signal in that context, wiring up its dependencies and subscribers
        let value = rc.run_in(&mut f);
        let recompute = RefCell::new(Box::new(f) as Box<dyn FnMut() -> T>);
        let update = CopyValue::new(UpdateInformation {
            dirty,
            callback: recompute,
        });
        let state: Signal<T> = Signal::new(value);

        let memo = Memo {
            inner: state,
            update,
        };

        spawn(async move {
            while rx.next().await.is_some() {
                memo.recompute();
            }
        });

        memo
    }

    /// Rerun the computation and update the value of the memo if the result has changed.
    fn recompute(&self)
    where
        T: PartialEq,
    {
        let mut update_copy = self.update;
        let update_write = update_copy.write();
        let peak = self.inner.peek();
        let new_value = (update_write.callback.borrow_mut())();
        if new_value != *peak {
            drop(peak);
            let mut copy = self.inner;
            let mut write = copy.write();
            *write = new_value;
            update_write
                .dirty
                .store(false, std::sync::atomic::Ordering::Relaxed);
        }
    }

    /// Get the scope that the signal was created in.
    pub fn origin_scope(&self) -> ScopeId {
        self.inner.origin_scope()
    }

    /// Get the id of the signal.
    pub fn id(&self) -> generational_box::GenerationalBoxId {
        self.inner.id()
    }
}

impl<T> Readable for Memo<T>
where
    T: PartialEq,
{
    type Target = T;
    type Storage = UnsyncStorage;

    #[track_caller]
    fn try_read(&self) -> Result<ReadableRef<Self>, generational_box::BorrowError> {
        let read = self.inner.try_read();
        match read {
            Ok(r) => {
                let needs_update = self
                    .update
                    .read()
                    .dirty
                    .swap(false, std::sync::atomic::Ordering::Relaxed);
                if needs_update {
                    drop(r);
                    self.recompute();
                    self.inner.try_read()
                } else {
                    Ok(r)
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Get the current value of the signal. **Unlike read, this will not subscribe the current scope to the signal which can cause parts of your UI to not update.**
    ///
    /// If the signal has been dropped, this will panic.
    #[track_caller]
    fn peek(&self) -> ReadableRef<Self> {
        self.inner.peek()
    }
}

impl<T> IntoAttributeValue for Memo<T>
where
    T: Clone + IntoAttributeValue + PartialEq,
{
    fn into_value(self) -> dioxus_core::AttributeValue {
        self.with(|f| f.clone().into_value())
    }
}

impl<T: 'static> PartialEq for Memo<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: Clone> Deref for Memo<T>
where
    T: PartialEq,
{
    type Target = dyn Fn() -> T;

    fn deref(&self) -> &Self::Target {
        Readable::deref_impl(self)
    }
}
