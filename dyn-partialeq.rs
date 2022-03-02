type BoxedObj = Box<dyn Obj>;

// AnyBoxedObj -----------------------------------------------------------------

trait AnyBoxedObj {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any>;
}

impl<T> AnyBoxedObj for T
where
    T: 'static + Obj,
{
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    
    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
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
        }
        else {
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

// Obj -------------------------------------------------------------------------

trait Obj: AnyBoxedObj + PartialEqBoxedObj + std::any::Any + std::fmt::Debug {}

// Val -------------------------------------------------------------------------

#[derive(Debug, PartialEq)]
enum Val {
    Int(i64),
    Obj(Box<dyn Obj>)
}

impl Val {
    /// Return reference to object of type T.
    pub fn obj<T: std::any::Any>(&self) -> Option<&T> {
        if let Val::Obj(obj) = self {
            return obj.as_any().downcast_ref::<T>()
        }

        None
    }
    
    /// Return mutable reference to object of type T.
    pub fn obj_mut<T: std::any::Any>(&mut self) -> Option<&mut T> {
        if let Val::Obj(obj) = self {
            return obj.as_any_mut().downcast_mut::<T>()
        }

        None
    }
    
    /// Extract object of type T from Val.
    pub fn obj_into<T: std::any::Any>(self) -> Option<T> {
        if let Val::Obj(obj) = self {
            if let Ok(inner) = obj.into_any().downcast::<T>() {
                return Some(*inner)
            }
        }

        None
    }
}


// Obj Complex -----------------------------------------------------------------

#[derive(Debug, PartialEq)]
struct Complex {
    x: i64,
    y: i64
}

impl Obj for Complex {}


impl From<i64> for Val {
    fn from(value: i64) -> Self {
        Val::Int(value)
    }
}

impl From<Complex> for Val {
    fn from(value: Complex) -> Self {
        Val::Obj(Box::new(value))
    }
}

// main ------------------------------------------------------------------------

fn main() {
    // Create an int
    let ival = Val::from(1337);
    println!("ival = {:?}", ival);

    // Create the complex
    let complex = Complex{ x: 23, y: 42 };
    println!("complex = {:?}", complex);
    
    // Create another complex
    let complex2 = Complex{ x: 23, y: 42 };
    println!("complex2 = {:?}", complex2);
    
    // Turn complex into a val
    let mut val = Val::from(complex);
    println!("val = {:?}", val);
    
    let val2 = Val::from(complex2);
    println!("val2 = {:?}", val2);
    
    // Compare stuff
    println!("val == val2 = {}", val == val2);
    println!("val == ival = {}", val == ival);
    
    // Get reference to complex inside of the object.
    {
        let cp = val.obj::<Complex>().unwrap();
        println!("cp = {:?}", cp);
    }
    
    // Get mutable reference to complex inside of the object.
    {
        let mut cp = val.obj_mut::<Complex>().unwrap();
        cp.x = 1337;
        cp.y = 666;
        println!("cp = {:?}", cp);
    }
    
    // Compare stuff
    println!("val == val2 = {}", val == val2);
    println!("val == ival = {}", val == ival);
    
    // Get the object, consuming the val.
    {
        let c = val.obj_into::<Complex>();
        println!("c = {:?}", c);
    }
}

