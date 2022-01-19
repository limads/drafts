use std::rc::Rc;
use std::cell::RefCell;
use std::boxed;

pub mod ui;

pub mod manager;

pub mod typesetter;

pub type Callbacks<T> = Rc<RefCell<Vec<boxed::Box<dyn Fn(T) + 'static>>>>;

pub type ValuedCallbacks<A, R> = Rc<RefCell<Vec<boxed::Box<dyn Fn(A)->R + 'static>>>>;

pub trait React<S> {

    fn react(&self, source : &S);

}
