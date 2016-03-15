pub trait Observer : Sized{
    type Item;
	fn on_next(self, value: Self::Item) -> Option<Self>;
	fn on_completed(self);
	// Not sure how the error stuff looks in Rust
	// fn on_error(Self/*,???*/);
}

pub trait Observable: Sized {
    type Item;
    type Subscription : Drop;
    fn subscribe<Q: Observer<Item = Self::Item>>(self, o: Q) -> Self::Subscription;
    fn filter<F: FnMut(&Self::Item) -> bool>(self, f: F) -> filter::Filter<Self, F> {
        filter::new(self, f)
    }

    fn map<T, F: FnMut(Self::Item) -> T>(self, f: F) -> map::Map<T, Self, F> {
        map::new(self, f)
    }

    fn take(self, count: u64) -> take::Take<Self> {
        take::new(self, count)
    }

    fn take_while<F: FnMut(&Self::Item) -> bool>(self, f: F) -> take_while::TakeWhile<Self, F> {
        take_while::new(self, f)
    }

    fn take_until<F: FnMut(&Self::Item) -> bool>(self, f: F) -> take_until::TakeUntil<Self, F> {
        take_until::new(self, f)
    }
    
    fn concat<O2: Observable<Item = Self::Item>>(self, other: O2) -> concat::Concat<Self, O2> {
        concat::new(self, other)
    }
}

pub mod filter {
    use super::{Observer, Observable};
    pub struct Filter<O: Observable, F: FnMut(&O::Item) -> bool> { f: F, inner: O }
    impl<O: Observable, F: FnMut(&O::Item) ->bool> Observable for Filter<O, F> {
        type Item = O::Item;
        type Subscription = O::Subscription;
        fn subscribe<Q: Observer<Item = Self::Item>,>(self, o: Q) -> Self::Subscription {
            let observer = FilterObserver { f: self.f, inner: o };
            self.inner.subscribe(observer)
        }
    }
    pub struct FilterObserver<Q: Observer, F: FnMut(&Q::Item) -> bool> { f: F, inner: Q }

    impl<Q: Observer, F: FnMut(&Q::Item) -> bool> Observer for FilterObserver<Q, F> {
        type Item = Q::Item;
        fn on_next(mut self, value: Q::Item) -> Option<Self> {
            if (self.f)(&value) {
                let f = self.f;
                self.inner.on_next(value).map(|o| { FilterObserver { inner: o, f: f }})
            } else { Some(self) }
        }
        fn on_completed(self) {
            self.inner.on_completed();
        }
    }

    pub fn new<O: Observable, F: FnMut(&O::Item) -> bool>(seq: O, f: F) -> Filter<O, F> {
        
        Filter { inner: seq, f: f }
    }
    // add code here
}

pub fn filter<O, F>(seq: O, f: F) -> filter::Filter<O, F>
    where O: Observable, F: FnMut(&O::Item) -> bool {
    filter::new(seq,f)
}

pub mod map {
    use std::marker::PhantomData;
    use super::{Observer, Observable};
    pub struct Map<T, O: Observable, F: FnMut(O::Item) -> T> { f: F, inner: O, _t: PhantomData<T> }
    impl<T, O: Observable, F: FnMut(O::Item) -> T> Observable for Map<T, O, F> {
        type Item = T;
        type Subscription = O::Subscription;
        fn subscribe<Q: Observer<Item = Self::Item>>(self, o: Q) -> Self::Subscription {
            let observer = MapObserver { f: self.f, inner: o, _t: PhantomData };
            self.inner.subscribe(observer)
        }
    }
    
    pub struct MapObserver<T, Q: Observer, F : FnMut(T) -> Q::Item> { f: F, inner: Q, _t: PhantomData<T> }

    impl<T, Q: Observer, F: FnMut(T) -> Q::Item> Observer for MapObserver<T, Q, F> {
        type Item = T;
        fn on_next(self, value: T) -> Option<Self> {
            let mut f = self.f;
            let value = f(value);
            self.inner.on_next(value).map(|o| { MapObserver { inner: o, f: f, _t: PhantomData } })
        }
        fn on_completed(self) {
            self.inner.on_completed();
        }
    }
       
    pub fn new<T, O: Observable, F: FnMut(O::Item) -> T>(inner: O, f: F) -> Map<T, O, F> {
        Map { f: f, inner: inner, _t: PhantomData }
    }
}
pub struct DoNothingSub;
impl Drop for DoNothingSub {
    fn drop(&mut self) {}
}
pub mod never {
    use std::marker::PhantomData;
    use super::{Observable, Observer, DoNothingSub};
    pub struct Never<T> { _t: PhantomData<T> }
    impl<T> Observable for Never<T> {
        type Item = T;
        type Subscription = DoNothingSub;
        fn subscribe<Q: Observer<Item = T>>(self, _o: Q) -> Self::Subscription {
            DoNothingSub
        }
    }

    pub fn new<T>() -> Never<T> { Never { _t: PhantomData } }
}
pub fn never<T>() -> never::Never<T> { 
    never::new::<T>()
}

pub mod empty {
    use std::marker::PhantomData;
    use super::{Observable, Observer, DoNothingSub};
    pub struct Empty<T> { _t: PhantomData<T> }
    impl<T> Observable for Empty<T> {
        type Item = T;
        type Subscription = DoNothingSub;
        fn subscribe<Q: Observer<Item = T>>(self, o: Q) -> Self::Subscription {
            o.on_completed();
            DoNothingSub
        }
    }
    pub fn new<T>() -> Empty<T> { Empty { _t: PhantomData } }
}
pub mod take {
    use super::{Observable, Observer};
    pub struct Take<O: Observable> { inner: O, count: u64 }
    pub struct TakeObserver<Q: Observer> { inner: Q, count: u64 }
    impl<Q: Observer> Observer for TakeObserver<Q> {
        type Item = Q::Item;
        fn on_next(mut self, val: Q::Item) -> Option<Self> {
            if self.count > 0 {
                let o = self.inner.on_next(val);
                if o.is_none() {
                    return None;
                }
                self.inner = o.unwrap();
                self.count -= 1;
                if self.count == 0 {
                    self.inner.on_completed();
                    return None;
                }
            }
            Some(self)
        }

        fn on_completed(self) {
            self.inner.on_completed();
        }
    }
    impl<O: Observable> Observable for Take<O> {
        type Item = O::Item;
        type Subscription = O::Subscription;
        fn subscribe<Q: Observer<Item = Self::Item>>(self, observer: Q) -> Self::Subscription {
            self.inner.subscribe(TakeObserver { inner: observer, count: self.count })
        }
    }
    pub fn new<O: Observable>(inner: O, count: u64) -> Take<O> {
        Take { inner: inner, count: count }
    }
}

pub mod take_until {
    use super::{Observable, Observer};
    pub struct TakeUntil<O: Observable, F: FnMut(&O::Item) -> bool> { inner: O, f: F }
    pub struct TakeUntilObserver<Q: Observer, F: FnMut(&Q::Item) -> bool> { inner: Q, f: F }
    impl<Q: Observer, F: FnMut(&Q::Item) -> bool> Observer for TakeUntilObserver<Q, F> {
        type Item = Q::Item;
        fn on_next(mut self, val: Self::Item) -> Option<Self> {
            let is_end = (self.f)(&val);
            let o = self.inner.on_next(val);
            if o.is_none() {
                return None;
            }
            self.inner = o.unwrap();
            if is_end {
                self.inner.on_completed();
                return None;
            }
            Some(self)
        }

        fn on_completed(self) {
            self.inner.on_completed();
        }
    }
    impl<O: Observable, F: FnMut(&O::Item) -> bool> Observable for TakeUntil<O, F> {
        type Item = O::Item;
        type Subscription = O::Subscription;
        fn subscribe<Q: Observer<Item = Self::Item>>(self, observer: Q) -> Self::Subscription {
            self.inner.subscribe(TakeUntilObserver { inner: observer, f: self.f })
        }
    }
    pub fn new<O: Observable, F: FnMut(&O::Item) -> bool>(inner: O, f: F) -> TakeUntil<O, F> {
        TakeUntil { inner: inner, f: f }
    }
}

pub mod take_while {
    use super::{Observable, Observer};
    pub struct TakeWhile<O: Observable, F: FnMut(&O::Item) -> bool> { inner: O, f: F }
    pub struct TakeWhileObserver<Q: Observer, F: FnMut(&Q::Item) -> bool> { inner: Q, f: F }
    impl<Q: Observer, F: FnMut(&Q::Item) -> bool> Observer for TakeWhileObserver<Q, F> {
        type Item = Q::Item;
        fn on_next(mut self, val: Q::Item) -> Option<Self> {
            if (self.f)(&val) {
                if let Some(o) = self.inner.on_next(val) {
                    self.inner = o;
                    Some(self)
                } else {
                    None
                }
            } else {
                self.inner.on_completed();
                None
            }
        }

        fn on_completed(self) {
            self.inner.on_completed();
        }
    }
    impl<O: Observable, F: FnMut(&O::Item) -> bool> Observable for TakeWhile<O, F> {
        type Item = O::Item;
        type Subscription = O::Subscription;
        fn subscribe<Q: Observer<Item = Self::Item>>(self, observer: Q) -> Self::Subscription {
            self.inner.subscribe(TakeWhileObserver { inner: observer, f: self.f })
        }
    }

    pub fn new<O: Observable, F: FnMut(&O::Item) -> bool>(o: O, f: F) -> TakeWhile<O, F> {
        TakeWhile { inner: o, f: f }
    }
}

pub mod skip {
    use super::{Observable, Observer};
    pub struct Skip<O: Observable> { inner: O, count: u64 }

    impl<O: Observable> Observable for Skip<O> {
        type Item = O::Item;
        type Subscription = O::Subscription;
        fn subscribe<Q: Observer<Item = Self::Item>>(self, observer: Q) -> Self::Subscription {
            self.inner.subscribe(SkipObserver { inner: observer, count: self.count })
        }
    }


    pub struct SkipObserver<Q: Observer> { inner: Q, count: u64 }
    impl<Q: Observer> Observer for SkipObserver<Q> {
        type Item = Q::Item;
        fn on_next(mut self, val: Q::Item) -> Option<Self> {
            if self.count == 0 {
                if let Some(next) = self.inner.on_next(val) {
                self.inner = next;
                } else {
                    return None;
                }
            } else {
                self.count -= 1;
            }
            Some(self)
        }

        fn on_completed(self) {
            self.inner.on_completed();
        }
    }
}

pub mod skip_while {
    use super::{Observable, Observer};
    pub struct SkipWhile<O: Observable, F: FnMut(&O::Item) -> bool> { inner: O, f: F }
    pub struct SkipWhileObserver<Q : Observer, F: FnMut(&Q::Item) -> bool> { inner: Q, f: Option<F> }

    impl<O: Observable, F: FnMut(&O::Item) -> bool> Observable for SkipWhile<O, F> {
        type Item = O::Item;
        type Subscription = O::Subscription;
        fn subscribe<Q: Observer<Item = Self::Item>>(self, observer: Q) -> Self::Subscription {
            self.inner.subscribe(SkipWhileObserver { inner: observer, f: Some(self.f) })
        }
    }
    
    impl<Q: Observer, F: FnMut(&Q::Item) -> bool> Observer for SkipWhileObserver<Q, F> {
        type Item = Q::Item;
        fn on_next(mut self, val: Q::Item) -> Option<Self> {
            if let Some(mut f) = self.f {
                if f(&val) {
                    self.f = Some(f);
                    return Some(self);
                }
                self.f = None;
            } 
            if let Some(next) = self.inner.on_next(val) {
                self.inner = next;
                Some(self)
            } else {
                None
            }
        }

        fn on_completed(self) {
            self.inner.on_completed();
        }
    }
}

pub mod concat {
    use std::sync::{Arc, Weak, Mutex};
    use std::mem;
    use super::{Observer, Observable};
    pub struct Concat<O1, O2> { seq1: O1, seq2: O2 }
    pub struct ConcatObserver<Q, O2: Observable, Sub1> { observer: Q, seq2: O2, sub: Weak<Mutex<EitherSub<Sub1, O2::Subscription>>> }

    enum EitherSub<Sub1, Sub2> {
        Initial,
        Sub1(Sub1),
        Sub2(Sub2),
        Done
    }

    pub struct SwitchingSubscription<Sub1, Sub2>(Arc<Mutex<EitherSub<Sub1, Sub2>>>);

    impl<Sub1, Sub2> Drop for SwitchingSubscription<Sub1, Sub2> {
        fn drop(&mut self) {
            let mut sub = EitherSub::Done;
            {
                sub = mem::replace(&mut *self.0.lock().unwrap(), sub);
            }
        }
    }
    impl<Q: Observer, O2: Observable<Item = Q::Item>, Sub1> Observer for ConcatObserver<Q, O2, Sub1> {
        type Item = Q::Item;
        fn on_next(mut self, value: Self::Item) -> Option<Self> {
            if let Some(next) = self.observer.on_next(value) {
                self.observer = next;
                Some(self)
            } else {
                None
            }
        }
        fn on_completed(self) {
            if let Some(submutex) = self.sub.upgrade() {
                if let EitherSub::Done = *submutex.lock().unwrap() {
                    return;
                }
                let next_sub = self.seq2.subscribe(self.observer);
                let mut sub_releaser = EitherSub::Sub2(next_sub);
                {
                    let mut subplace = submutex.lock().unwrap();
                    if let EitherSub::Done = *subplace {
                        return;
                    }
                    sub_releaser = mem::replace(&mut *subplace, sub_releaser);
                }

            }
        }
    }
    impl<O1: Observable, O2: Observable<Item = O1::Item>> Observable for Concat<O1, O2> {
        type Item = O1::Item;
        type Subscription = SwitchingSubscription<O1::Subscription, O2::Subscription>;
        fn subscribe<Q: Observer<Item = Self::Item>>(self, observer: Q) -> Self::Subscription {
            let sub_holder = Arc::new(Mutex::new(EitherSub::Initial as EitherSub<O1::Subscription, O2::Subscription>));
            let weak_sub = Arc::downgrade(&sub_holder);
            let o = ConcatObserver { observer: observer, seq2: self.seq2, sub: weak_sub };
            let sub = self.seq1.subscribe(o);
            {
                let mut holder = sub_holder.lock().unwrap();
                if let EitherSub::Initial = *holder {
                    *holder = EitherSub::Sub1(sub);
                }
            }
            SwitchingSubscription(sub_holder)
        }
    }
    pub fn new<O1: Observable, O2: Observable<Item = O1::Item>>(seq1: O1, seq2: O2) -> Concat<O1, O2> {
        Concat { seq1: seq1, seq2: seq2 }
    }
}

pub mod from_iter {
    use std::iter::IntoIterator;
    use super::{Observer, Observable};

    pub struct TestSequence<I: IntoIterator> { it: I }
    impl<I: IntoIterator> Observable for TestSequence<I> {
        type Item = I::Item;
        type Subscription = Sub;
        fn subscribe<Q: Observer<Item = Self::Item>>(self, mut o: Q) -> Self::Subscription {
            for x in self.it {
                if let Some(o2) = o.on_next(x) {
                    o = o2;
                } else {
                    return Sub;
                }
            }
            o.on_completed();
            Sub
        }

    }

    pub struct Sub;
    impl Drop for Sub { fn drop(&mut self) {} }

    pub fn from_iter<I: IntoIterator>(it: I) -> TestSequence<I> { TestSequence { it: it } }
}

pub use self::from_iter::from_iter;