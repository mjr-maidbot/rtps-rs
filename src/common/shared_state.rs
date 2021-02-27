use std::{
    cell::RefCell,
    marker::PhantomData,
    rc::Rc,
    sync::{
        Arc,
        RwLock,
    },
};

pub trait SharedState<T> : Clone {
    fn new(state: T) -> Self;
    fn call<R, F: Fn(&T) -> R>(&self, f: F) -> R; 
    fn call_mut<R, F: FnOnce(&mut T) -> R>(&self, f: F) -> R; 
}

impl<T> SharedState<T> for Rc<RefCell<T>> {
    fn new(state: T) -> Self {
        Rc::new(RefCell::new(state))
    }
    fn call<R, F: Fn(&T) -> R>(&self, f: F) -> R {
        f(&*self.borrow())
    }
    fn call_mut<R, F: FnOnce(&mut T) -> R>(&self, mut f: F) -> R {
        f(&mut *self.borrow_mut())
    }
}

impl<T> SharedState<T> for Arc<RwLock<T>> {
    fn new(t: T) -> Self {
        Arc::new(RwLock::new(t))
    }
    fn call<R, F: Fn(&T) -> R>(&self, f: F) -> R {
        f(&*self.read().unwrap())
    }
    fn call_mut<R, F: FnOnce(&mut T) -> R>(&self, mut f: F) -> R {
        f(&mut *self.write().unwrap())
    }
}
