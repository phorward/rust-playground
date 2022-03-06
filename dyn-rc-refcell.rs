use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

type BoxedObj = Box<dyn Obj>;

// AnyBoxedObj -----------------------------------------------------------------

trait AnyBoxedObj {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T> AnyBoxedObj for T
where
    T: 'static + Obj,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

// PartialEqBoxedObj -----------------------------------------------------------

trait PartialEqBoxedObj {
    fn dyn_eq(&self, other: &BoxedObj) -> bool;
}

impl<T> PartialEqBoxedObj for T
where
    T: 'static + Obj + PartialEq,
{
    fn dyn_eq(&self, other: &BoxedObj) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<T>() {
            self.eq(other)
        } else {
            false
        }
    }
}

impl PartialEq for BoxedObj {
    fn eq(&self, other: &Self) -> bool {
        self.dyn_eq(other)
    }
}

// fix for `move occurs because `*__arg_1_0` has type `Box<dyn Obj>`, which does not implement the `Copy` trait`
// https://github.com/rust-lang/rust/issues/31740#issuecomment-700950186
impl PartialEq<&Self> for BoxedObj {
    fn eq(&self, other: &&Self) -> bool {
        self.dyn_eq(other)
    }
}

// PartialOrdBoxedObj ----------------------------------------------------------

trait PartialOrdBoxedObj {
    fn dyn_partial_cmp(&self, other: &BoxedObj) -> Option<std::cmp::Ordering>;
}

impl<T> PartialOrdBoxedObj for T
where
    T: 'static + Obj + PartialEq + PartialOrd,
{
    fn dyn_partial_cmp(&self, other: &BoxedObj) -> Option<std::cmp::Ordering> {
        if let Some(other) = other.as_any().downcast_ref::<T>() {
            self.partial_cmp(other)
        } else {
            None
        }
    }
}

impl PartialOrd for BoxedObj {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.dyn_partial_cmp(other)
    }
}

// Obj -------------------------------------------------------------------------

trait Obj: AnyBoxedObj + PartialEqBoxedObj + PartialOrdBoxedObj + Any + std::fmt::Debug {}

// Val -------------------------------------------------------------------------

#[derive(Debug, PartialEq, PartialOrd)]
struct Val {
    value: Rc<RefCell<BoxedObj>>,
}

impl Val {
    pub fn new(value: BoxedObj) -> Self {
        Val {
            value: Rc::new(RefCell::new(value)),
        }
    }

    /// Return reference to object of type T.
    pub fn object<T: Any>(&self) -> Option<Ref<T>> {
        let value = self.value.borrow();
        if value.as_any().is::<T>() {
            return Some(Ref::map(value, |obj| {
                obj.as_any().downcast_ref::<T>().unwrap()
            }));
        }

        None
    }

    /// Return mutable reference to object of type T.
    pub fn object_mut<T: Any>(&self) -> Option<RefMut<T>> {
        let value = self.value.borrow_mut();
        if value.as_any().is::<T>() {
            return Some(RefMut::map(value, |obj| {
                obj.as_any_mut().downcast_mut::<T>().unwrap()
            }));
        }

        None
    }

    /// Return mutable reference to object of type T.
    pub fn into_object<T: Any>(self) -> Option<T> {
        let value = Rc::try_unwrap(self.value).unwrap().into_inner();
        if let Ok(inner) = value.into_any().downcast::<T>() {
            return Some(*inner);
        }

        None
    }
}

// Obj Int ---------------------------------------------------------------------
#[derive(Debug, PartialEq, PartialOrd)]
struct Int {
    int: i64,
}

impl Obj for Int {}

impl From<i64> for Val {
    fn from(int: i64) -> Self {
        Val::new(Box::new(Int { int }))
    }
}

// Obj Complex -----------------------------------------------------------------
#[derive(Debug, PartialEq, PartialOrd)]
struct Complex {
    x: i64,
    y: i64,
}

impl Obj for Complex {}

impl From<Complex> for Val {
    fn from(value: Complex) -> Self {
        Val::new(Box::new(value))
    }
}

// main ------------------------------------------------------------------------

fn main() {
    // Create an int
    let ival = Val::from(1337);
    println!("ival = {:?}", ival);

    // Create the complex
    let complex = Complex { x: 23, y: 42 };
    println!("complex = {:?}", complex);

    // Create another complex
    let complex2 = Complex { x: 23, y: 42 };
    println!("complex2 = {:?}", complex2);

    // Turn complex into a val
    let mut val = Val::from(complex);
    println!("val = {:?}", val);

    let val2 = Val::from(complex2);
    println!("val2 = {:?}", val2);

    // Compare stuff
    println!("val == val2 = {}", val == val2);
    println!("val < val2 = {}", val < val2);
    println!("val > val2 = {}", val > val2);
    println!("val <= val2 = {}", val <= val2);
    println!("val >= val2 = {}", val >= val2);
    println!("val == ival = {}", val == ival);

    // Get reference to complex inside of the object.
    {
        let cp = val.object::<Complex>().unwrap();
        println!("cp = {:?}", cp);
    }

    // Get mutable reference to complex inside of the object.
    {
        let mut cp = val.object_mut::<Complex>().unwrap();
        cp.x = 1337;
        cp.y = 666;
        println!("cp = {:?}", cp);
    }

    // Compare stuff
    println!("val == val2 = {}", val == val2);
    println!("val < val2 = {}", val < val2);
    println!("val > val2 = {}", val > val2);
    println!("val <= val2 = {}", val <= val2);
    println!("val >= val2 = {}", val >= val2);
    println!("val == ival = {}", val == ival);

    // Get the object, consuming the val.
    {
        let c = val.into_object::<Complex>();
        println!("c = {:?}", c);
    }
}
