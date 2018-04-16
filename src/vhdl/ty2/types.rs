// Copyright (c) 2017-2018 Fabian Schuiki

//! Dealing with types in an abstract manner.

use std::fmt::{self, Debug, Display};
use std::iter::{once, repeat};
use std::ops::{Add, Sub, Deref};

pub use num::BigInt;
use num::One;

use common::name::{get_name_table, Name};

/// An interface for dealing with types.
///
/// This is the main type trait, which all types and subtypes implement.
pub trait Type: Debug + Display {
    /// Check if this is a scalar type.
    ///
    /// Enumeration, integer, floating-point, and physical types are scalar.
    fn is_scalar(&self) -> bool;

    /// Check if this is a discrete type.
    ///
    /// Enumeration and integer types are discrete.
    fn is_discrete(&self) -> bool;

    /// Check if this is a numeric type.
    ///
    /// Integer, floating-point, and physical types are numeric.
    fn is_numeric(&self) -> bool;

    /// Check if this is a composite type.
    ///
    /// Array and record types are composite.
    fn is_composite(&self) -> bool;

    /// Converts from `&Type` to `AnyType`.
    fn as_any(&self) -> AnyType;
}

/// A type.
///
/// This enum represents one of the types declared in this module. It is useful
/// in code that needs to know exactly what type it is operating on, for example
/// in a match expression. This is the root of the entire type system. If a user
/// declares a type, this enum carries the information as to which type was
/// declared.
#[derive(Copy, Clone)]
#[allow(missing_docs)]
pub enum AnyType<'t> {
    Enum(&'t EnumType),
    Integer(&'t IntegerType),
    Floating(&'t FloatingType),
    Physical(&'t PhysicalType),
    Array(&'t ArrayType<'t>),
    // record
    // access
    // file
    // protected

    // Non-standard types.
    Null,
    UniversalInteger,
    UniversalReal,
    // subprogram
}

impl<'t> Type for AnyType<'t> {
    fn is_scalar(&self) -> bool {
        match *self {
            AnyType::Enum(t)          => t.is_scalar(),
            AnyType::Integer(t)       => t.is_scalar(),
            AnyType::Floating(t)      => t.is_scalar(),
            AnyType::Physical(t)      => t.is_scalar(),
            AnyType::Array(t)         => t.is_scalar(),
            AnyType::Null             => NullType.is_scalar(),
            AnyType::UniversalInteger => UniversalIntegerType.is_scalar(),
            AnyType::UniversalReal    => UniversalRealType.is_scalar(),
        }
    }

    fn is_discrete(&self) -> bool {
        match *self {
            AnyType::Enum(t)          => t.is_discrete(),
            AnyType::Integer(t)       => t.is_discrete(),
            AnyType::Floating(t)      => t.is_discrete(),
            AnyType::Physical(t)      => t.is_discrete(),
            AnyType::Array(t)         => t.is_discrete(),
            AnyType::Null             => NullType.is_discrete(),
            AnyType::UniversalInteger => UniversalIntegerType.is_discrete(),
            AnyType::UniversalReal    => UniversalRealType.is_discrete(),
        }
    }

    fn is_numeric(&self) -> bool {
        match *self {
            AnyType::Enum(t)          => t.is_numeric(),
            AnyType::Integer(t)       => t.is_numeric(),
            AnyType::Floating(t)      => t.is_numeric(),
            AnyType::Physical(t)      => t.is_numeric(),
            AnyType::Array(t)         => t.is_numeric(),
            AnyType::Null             => NullType.is_numeric(),
            AnyType::UniversalInteger => UniversalIntegerType.is_numeric(),
            AnyType::UniversalReal    => UniversalRealType.is_numeric(),
        }
    }

    fn is_composite(&self) -> bool {
        match *self {
            AnyType::Enum(t)          => t.is_composite(),
            AnyType::Integer(t)       => t.is_composite(),
            AnyType::Floating(t)      => t.is_composite(),
            AnyType::Physical(t)      => t.is_composite(),
            AnyType::Array(t)         => t.is_composite(),
            AnyType::Null             => NullType.is_composite(),
            AnyType::UniversalInteger => UniversalIntegerType.is_composite(),
            AnyType::UniversalReal    => UniversalRealType.is_composite(),
        }
    }

    fn as_any(&self) -> AnyType {
        *self
    }
}

impl<'t> Display for AnyType<'t> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AnyType::Enum(t)          => Display::fmt(t, f),
            AnyType::Integer(t)       => Display::fmt(t, f),
            AnyType::Floating(t)      => Display::fmt(t, f),
            AnyType::Physical(t)      => Display::fmt(t, f),
            AnyType::Array(t)         => Display::fmt(t, f),
            AnyType::Null             => Display::fmt(&NullType, f),
            AnyType::UniversalInteger => Display::fmt(&UniversalIntegerType, f),
            AnyType::UniversalReal    => Display::fmt(&UniversalRealType, f),
        }
    }
}

impl<'t> Debug for AnyType<'t> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AnyType::Enum(t)          => Debug::fmt(t, f),
            AnyType::Integer(t)       => Debug::fmt(t, f),
            AnyType::Floating(t)      => Debug::fmt(t, f),
            AnyType::Physical(t)      => Debug::fmt(t, f),
            AnyType::Array(t)         => Debug::fmt(t, f),
            AnyType::Null             => Debug::fmt(&NullType, f),
            AnyType::UniversalInteger => Debug::fmt(&UniversalIntegerType, f),
            AnyType::UniversalReal    => Debug::fmt(&UniversalRealType, f),
        }
    }
}

impl<'t, T: Type> From<&'t T> for AnyType<'t> {
    fn from(ty: &'t T) -> AnyType<'t> {
        ty.as_any()
    }
}

impl<'t> AnyType<'t> {
    /// Returns `Some(t)` if the type is `Enum(t)`, `None` otherwise.
    pub fn as_enum(self) -> Option<&'t EnumType> {
        match self { AnyType::Enum(t) => Some(t), _ => None }
    }

    /// Returns `Some(t)` if the type is `Integer(t)`, `None` otherwise.
    pub fn as_integer(self) -> Option<&'t IntegerType> {
        match self { AnyType::Integer(t) => Some(t), _ => None }
    }

    /// Returns `Some(t)` if the type is `Floating(t)`, `None` otherwise.
    pub fn as_floating(self) -> Option<&'t FloatingType> {
        match self { AnyType::Floating(t) => Some(t), _ => None }
    }

    /// Returns `Some(t)` if the type is `Physical(t)`, `None` otherwise.
    pub fn as_physical(self) -> Option<&'t PhysicalType> {
        match self { AnyType::Physical(t) => Some(t), _ => None }
    }

    /// Returns `Some(t)` if the type is `Array(t)`, `None` otherwise.
    pub fn as_array(self) -> Option<&'t ArrayType<'t>> {
        match self { AnyType::Array(t) => Some(t), _ => None }
    }

    /// Checks if the type is `Null`.
    pub fn is_null(self) -> bool {
        match self { AnyType::Null => true, _ => false }
    }

    /// Checks if the type is `UniversalInteger`.
    pub fn is_universal_integer(self) -> bool {
        match self { AnyType::UniversalInteger => true, _ => false }
    }

    /// Checks if the type is `UniversalReal`.
    pub fn is_universal_real(self) -> bool {
        match self { AnyType::UniversalReal => true, _ => false }
    }

    /// Returns an `&EnumType` or panics if the type is not `Enum`.
    pub fn unwrap_enum(self) -> &'t EnumType {
        self.as_enum().expect("type is not an enum")
    }

    /// Returns an `&IntegerType` or panics if the type is not `Integer`.
    pub fn unwrap_integer(self) -> &'t IntegerType {
        self.as_integer().expect("type is not an integer")
    }

    /// Returns an `&FloatingType` or panics if the type is not `Floating`.
    pub fn unwrap_floating(self) -> &'t FloatingType {
        self.as_floating().expect("type is not an floating")
    }

    /// Returns an `&PhysicalType` or panics if the type is not `Physical`.
    pub fn unwrap_physical(self) -> &'t PhysicalType {
        self.as_physical().expect("type is not an physical")
    }

    /// Returns an `&ArrayType` or panics if the type is not `Array`.
    pub fn unwrap_array(self) -> &'t ArrayType<'t> {
        self.as_array().expect("type is not an array")
    }
}

/// An enumeration type.
#[derive(Debug)]
pub struct EnumType {
    /// The enumeration literals.
    lits: Vec<EnumLiteral>,
}

impl EnumType {
    /// Create a new enumeration type.
    ///
    /// # Example
    ///
    /// ```
    /// use moore_vhdl::ty2::{Type, EnumType};
    ///
    /// let ty = EnumType::new(vec![
    ///     "first".into(),
    ///     "second".into(),
    ///     '0'.into(),
    ///     '1'.into(),
    /// ]);
    ///
    /// assert_eq!(format!("{}", ty), "(first, second, '0', '1')");
    /// ```
    pub fn new<I: IntoIterator<Item=EnumLiteral>>(lits: I) -> EnumType {
        EnumType {
            lits: lits.into_iter().collect(),
        }
    }

    /// The number of literals.
    pub fn len(&self) -> usize {
        self.lits.len()
    }

    /// A literal by position.
    pub fn literal(&self, pos: usize) -> &EnumLiteral {
        &self.lits[pos]
    }

    /// Return the literals.
    pub fn literals(&self) -> &[EnumLiteral] {
        &self.lits
    }
}

impl Type for EnumType {
    fn is_scalar(&self) -> bool { true }
    fn is_discrete(&self) -> bool { true }
    fn is_numeric(&self) -> bool { false }
    fn is_composite(&self) -> bool { false }
    fn as_any(&self) -> AnyType { AnyType::Enum(self) }
}

impl Display for EnumType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(")?;
        for (sep, lit) in once("").chain(repeat(", ")).zip(self.lits.iter()) {
            write!(f, "{}{}", sep, lit)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

/// An enumeration literal.
///
/// Distinguishes between:
/// - identifier literals such as `FOO`, and
/// - character literals such as `'0'`.
#[derive(Debug)]
pub enum EnumLiteral {
    /// An identifier enumeration literal.
    Ident(Name),
    /// A character enumeration ltieral.
    Char(char),
}

impl<'a> From<&'a str> for EnumLiteral {
    fn from(n: &'a str) -> EnumLiteral {
        EnumLiteral::Ident(get_name_table().intern(n, false))
    }
}

impl From<Name> for EnumLiteral {
    fn from(n: Name) -> EnumLiteral {
        EnumLiteral::Ident(n)
    }
}

impl From<char> for EnumLiteral {
    fn from(c: char) -> EnumLiteral {
        EnumLiteral::Char(c)
    }
}

impl Display for EnumLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EnumLiteral::Ident(n) => write!(f, "{}", n),
            EnumLiteral::Char(c) => write!(f, "'{}'", c),
        }
    }
}

/// An integer type.
///
/// This can either be an `IntegerBasetype` or an `IntegerSubtype`.
pub trait IntegerType: Type {
    /// The range of values this integer can assume.
    fn range(&self) -> &Range<BigInt>;

    /// The base type of this integer.
    fn base_type(&self) -> &Type;

    /// The resolution function associated with this type.
    fn resolution_func(&self) -> Option<usize> { None }
}

impl<'t, T> Type for T where T: IntegerType + 't {
    fn is_scalar(&self) -> bool { true }
    fn is_discrete(&self) -> bool { true }
    fn is_numeric(&self) -> bool { true }
    fn is_composite(&self) -> bool { false }
    fn as_any(&self) -> AnyType { AnyType::Integer(self) }
}

impl<'t> Deref for IntegerType + 't {
    type Target = Range<BigInt>;
    fn deref(&self) -> &Range<BigInt> {
        self.range()
    }
}

/// A floating-point type.
#[derive(Debug)]
pub struct FloatingType {
    /// The range of values.
    range: Range<f64>,
}

impl FloatingType {
    /// Create a new floating-point type.
    ///
    /// # Example
    ///
    /// ```
    /// use moore_vhdl::ty2::{Type, FloatingType, Range, RangeDir};
    ///
    /// let a = FloatingType::new(Range::ascending(0, 42));
    /// let b = FloatingType::new(Range::descending(42, 0));
    ///
    /// assert_eq!(format!("{}", a), "0 to 42");
    /// assert_eq!(format!("{}", b), "42 downto 0");
    /// assert_eq!(a.dir(), RangeDir::To);
    /// assert_eq!(b.dir(), RangeDir::Downto);
    /// assert_eq!(a.len(), f64::from(43));
    /// assert_eq!(b.len(), f64::from(43));
    /// ```
    pub fn new(range: Range<f64>) -> FloatingType {
        FloatingType {
            range: range,
        }
    }
}

impl Type for FloatingType {
    fn is_scalar(&self) -> bool { true }
    fn is_discrete(&self) -> bool { false }
    fn is_numeric(&self) -> bool { true }
    fn is_composite(&self) -> bool { false }
    fn as_any(&self) -> AnyType { AnyType::Floating(self) }
}

impl Display for FloatingType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.range)
    }
}

impl Deref for FloatingType {
    type Target = Range<f64>;
    fn deref(&self) -> &Range<f64> {
        &self.range
    }
}

/// A directed range of values.
///
/// `Range<T>` has the same semantics as ranges in VHDL. They have a direction
/// associated with them, and left and right bounds. The range may be a null
/// range if the lower bound is greater than or equal to the upper bound.
#[derive(Debug, PartialEq, Eq)]
pub struct Range<T> {
    /// The direction.
    dir: RangeDir,
    /// The left bound.
    left: T,
    /// The right bound.
    right: T,
}

impl<T: PartialOrd + One> Range<T> where for<'a> &'a T: Add<Output=T> + Sub<Output=T> {
    /// Create a range from left and right bounds.
    ///
    /// # Example
    ///
    /// ```
    /// use moore_vhdl::ty2::{IntegerRange, RangeDir};
    ///
    /// let a = IntegerRange::with_left_right(RangeDir::To, 0, 42);
    /// let b = IntegerRange::with_left_right(RangeDir::Downto, 42, 0);
    ///
    /// assert_eq!(format!("{}", a), "0 to 42");
    /// assert_eq!(format!("{}", b), "42 downto 0");
    /// ```
    pub fn with_left_right<D,L,R>(dir: D, left: L, right: R) -> Range<T>
        where RangeDir: From<D>, T: From<L> + From<R>
    {
        Range {
            dir: dir.into(),
            left: left.into(),
            right: right.into(),
        }
    }

    /// Create a range from lower and upper bounds.
    ///
    /// # Example
    ///
    /// ```
    /// use moore_vhdl::ty2::{IntegerRange, RangeDir};
    ///
    /// let a = IntegerRange::with_lower_upper(RangeDir::To, 0, 42);
    /// let b = IntegerRange::with_lower_upper(RangeDir::Downto, 0, 42);
    ///
    /// assert_eq!(format!("{}", a), "0 to 42");
    /// assert_eq!(format!("{}", b), "42 downto 0");
    /// ```
    pub fn with_lower_upper<D,L,U>(dir: D, lower: L, upper: U) -> Range<T>
        where RangeDir: From<D>, T: From<L> + From<U>
    {
        let dir = dir.into();
        let (left, right) = match dir {
            RangeDir::To => (lower.into(), upper.into()),
            RangeDir::Downto => (upper.into(), lower.into()),
        };
        Range {
            dir: dir,
            left: left,
            right: right,
        }
    }

    /// Create an ascending range.
    ///
    /// # Example
    ///
    /// ```
    /// use moore_vhdl::ty2::IntegerRange;
    ///
    /// let r = IntegerRange::ascending(0, 42);
    ///
    /// assert_eq!(format!("{}", r), "0 to 42");
    /// ```
    pub fn ascending<L,R>(left: L, right: R) -> Range<T> where T: From<L> + From<R> {
        Range {
            dir: RangeDir::To,
            left: left.into(),
            right: right.into(),
        }
    }

    /// Create a descending range.
    ///
    /// # Example
    ///
    /// ```
    /// use moore_vhdl::ty2::IntegerRange;
    ///
    /// let r = IntegerRange::descending(42, 0);
    ///
    /// assert_eq!(format!("{}", r), "42 downto 0");
    /// ```
    pub fn descending<L,R>(left: L, right: R) -> Range<T> where T: From<L> + From<R> {
        Range {
            dir: RangeDir::Downto,
            left: left.into(),
            right: right.into(),
        }
    }

    /// Return the direction of the range.
    ///
    /// # Example
    ///
    /// ```
    /// use moore_vhdl::ty2::{IntegerRange, RangeDir};
    ///
    /// let a = IntegerRange::ascending(0, 42);
    /// let b = IntegerRange::descending(42, 0);
    ///
    /// assert_eq!(a.dir(), RangeDir::To);
    /// assert_eq!(b.dir(), RangeDir::Downto);
    /// ```
    pub fn dir(&self) -> RangeDir {
        self.dir
    }

    /// Return the left bound of the range.
    ///
    /// # Example
    ///
    /// ```
    /// use moore_vhdl::ty2::{IntegerRange, BigInt};
    ///
    /// let a = IntegerRange::ascending(0, 42);
    /// let b = IntegerRange::descending(42, 0);
    ///
    /// assert_eq!(a.left(), &BigInt::from(0));
    /// assert_eq!(b.left(), &BigInt::from(42));
    /// ```
    pub fn left(&self) -> &T {
        &self.left
    }

    /// Return the right bound of the range.
    ///
    /// # Example
    ///
    /// ```
    /// use moore_vhdl::ty2::{IntegerRange, BigInt};
    ///
    /// let a = IntegerRange::ascending(0, 42);
    /// let b = IntegerRange::descending(42, 0);
    ///
    /// assert_eq!(a.right(), &BigInt::from(42));
    /// assert_eq!(b.right(), &BigInt::from(0));
    /// ```
    pub fn right(&self) -> &T {
        &self.right
    }

    /// Return the lower bound of the range.
    ///
    /// # Example
    ///
    /// ```
    /// use moore_vhdl::ty2::{IntegerRange, BigInt};
    ///
    /// let a = IntegerRange::ascending(0, 42);
    /// let b = IntegerRange::descending(42, 0);
    ///
    /// assert_eq!(a.lower(), &BigInt::from(0));
    /// assert_eq!(b.lower(), &BigInt::from(0));
    /// ```
    pub fn lower(&self) -> &T {
        match self.dir {
            RangeDir::To => &self.left,
            RangeDir::Downto => &self.right,
        }
    }

    /// Return the upper bound of the range.
    ///
    /// # Example
    ///
    /// ```
    /// use moore_vhdl::ty2::{IntegerRange, BigInt};
    ///
    /// let a = IntegerRange::ascending(0, 42);
    /// let b = IntegerRange::descending(42, 0);
    ///
    /// assert_eq!(a.upper(), &BigInt::from(42));
    /// assert_eq!(b.upper(), &BigInt::from(42));
    /// ```
    pub fn upper(&self) -> &T {
        match self.dir {
            RangeDir::To => &self.right,
            RangeDir::Downto => &self.left,
        }
    }

    /// Return true if the range is a null range.
    ///
    /// A null range has its lower bound greater than or equal to its upper
    /// bound, and thus also a length of 0 or lower.
    ///
    /// # Example
    ///
    /// ```
    /// use moore_vhdl::ty2::IntegerRange;
    ///
    /// let a = IntegerRange::ascending(0, 42);
    /// let b = IntegerRange::ascending(42, 0);
    ///
    /// assert_eq!(a.is_null(), false);
    /// assert_eq!(b.is_null(), true);
    /// ```
    pub fn is_null(&self) -> bool {
        self.lower() >= self.upper()
    }

    /// Return the length of the range.
    ///
    /// The length of a range is defined as `upper + 1 - lower`. The result may
    /// be negative, indicating that the range is a null range.
    ///
    /// # Example
    ///
    /// ```
    /// use moore_vhdl::ty2::{IntegerRange, BigInt};
    ///
    /// let a = IntegerRange::ascending(0, 42);
    /// let b = IntegerRange::ascending(42, 0);
    ///
    /// assert_eq!(a.len(), BigInt::from(43));
    /// assert_eq!(b.len(), BigInt::from(-41));
    /// ```
    pub fn len(&self) -> T {
        &(self.upper() + &One::one()) - self.lower()
    }

    /// Check if another range is a subrange of this range.
    ///
    /// This function checks if `self.lower()` is less than or equal to, and
    /// `self.upper()` is larger than or equal to, the corresponding bounds of
    /// the subrange.
    ///
    /// # Example
    ///
    /// ```
    /// use moore_vhdl::ty2::{IntegerRange, BigInt};
    ///
    /// let a = IntegerRange::ascending(0, 42);
    /// let b = IntegerRange::ascending(4, 16);
    /// let c = IntegerRange::descending(16, 4);
    ///
    /// assert_eq!(a.has_subrange(&b), true);
    /// assert_eq!(a.has_subrange(&c), true);
    /// assert_eq!(b.has_subrange(&a), false);
    /// assert_eq!(c.has_subrange(&a), false);
    /// assert_eq!(b.has_subrange(&c), true);
    /// assert_eq!(c.has_subrange(&b), true);
    /// ```
    pub fn has_subrange(&self, subrange: &Self) -> bool {
        self.lower() <= subrange.lower() && self.upper() >= subrange.upper()
    }
}

impl<T: Display> Display for Range<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {}", self.left, self.dir, self.right)
    }
}

/// A range of integer values.
pub type IntegerRange = Range<BigInt>;

/// A range of real values.
pub type RealRange = Range<f64>;

/// A range direction.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RangeDir {
    /// An ascending range.
    To,
    /// A descending range.
    Downto,
}

impl Display for RangeDir {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RangeDir::To => write!(f, "to"),
            RangeDir::Downto => write!(f, "downto"),
        }
    }
}

/// A physical type.
///
/// In VHDL a physical type is an integer multiple of some measurement unit.
/// A physical type has exactly one primary unit, and multiple secondary units
/// defined as multiples of that primary unit.
#[derive(Debug)]
pub struct PhysicalType {
    /// The range of integer multiples of the primary unit.
    range: Range<BigInt>,
    /// The units of this type.
    units: Vec<PhysicalUnit>,
    /// The index of the primary unit.
    primary: usize,
}

impl PhysicalType {
    /// Create a new physical type.
    ///
    /// # Example
    ///
    /// ```
    /// use moore_vhdl::ty2::{PhysicalType, PhysicalUnit, Range};
    /// use moore_vhdl::common::name::get_name_table;
    ///
    /// let ty = PhysicalType::new(Range::ascending(0, 1_000_000), vec![
    ///     PhysicalUnit::primary(get_name_table().intern("fs", false), 1),
    ///     PhysicalUnit::secondary(get_name_table().intern("ps", false), 1_000, 1000, 0),
    ///     PhysicalUnit::secondary(get_name_table().intern("ns", false), 1_000_000, 1000, 1),
    /// ], 0);
    ///
    /// assert_eq!(format!("{}", ty), "0 to 1000000 units (fs, ps, ns)");
    /// ```
    pub fn new<I>(range: Range<BigInt>, units: I, primary: usize) -> PhysicalType
        where I: IntoIterator<Item=PhysicalUnit>,
    {
        PhysicalType {
            range: range,
            units: units.into_iter().collect(),
            primary: primary,
        }
    }

    /// Return the units.
    pub fn units(&self) -> &[PhysicalUnit] {
        &self.units
    }

    /// Return the index of the primary unit.
    pub fn primary_index(&self) -> usize {
        self.primary
    }
}

impl Type for PhysicalType {
    fn is_scalar(&self) -> bool { true }
    fn is_discrete(&self) -> bool { false }
    fn is_numeric(&self) -> bool { true }
    fn is_composite(&self) -> bool { false }
    fn as_any(&self) -> AnyType { AnyType::Physical(self) }
}

impl Display for PhysicalType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} units (", self.range)?;
        for (sep, unit) in once("").chain(repeat(", ")).zip(self.units.iter()) {
            write!(f, "{}{}", sep, unit.name)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl Deref for PhysicalType {
    type Target = Range<BigInt>;
    fn deref(&self) -> &Range<BigInt> {
        &self.range
    }
}

/// A unit of a physical type.
#[derive(Debug, PartialEq, Eq)]
pub struct PhysicalUnit {
    /// The name of the unit.
    pub name: Name,
    /// The scale of the unit with respect to the physical type's primary unit.
    pub abs: BigInt,
    /// The scale of the unit with respect to another unit.
    pub rel: Option<(BigInt, usize)>,
}

impl PhysicalUnit {
    /// Create a new primary unit.
    ///
    /// # Example
    ///
    /// ```
    /// use moore_vhdl::ty2::{PhysicalUnit, BigInt};
    /// use moore_vhdl::common::name::get_name_table;
    ///
    /// let name = get_name_table().intern("fs", false);
    /// let unit = PhysicalUnit::primary(name, 1);
    ///
    /// assert_eq!(unit.name, name);
    /// assert_eq!(unit.abs, BigInt::from(1));
    /// assert_eq!(unit.rel, None);
    /// ```
    pub fn primary<A>(name: Name, abs: A) -> PhysicalUnit
        where BigInt: From<A>
    {
        PhysicalUnit {
            name: name,
            abs: abs.into(),
            rel: None,
        }
    }

    /// Create a new secondary unit.
    ///
    /// # Example
    ///
    /// ```
    /// use moore_vhdl::ty2::{PhysicalUnit, BigInt};
    /// use moore_vhdl::common::name::get_name_table;
    ///
    /// let name = get_name_table().intern("fs", false);
    /// let unit = PhysicalUnit::secondary(name, 1, 1000, 0);
    ///
    /// assert_eq!(unit.name, name);
    /// assert_eq!(unit.abs, BigInt::from(1));
    /// assert_eq!(unit.rel, Some((BigInt::from(1000), 0)));
    /// ```
    pub fn secondary<A,R>(name: Name, abs: A, rel: R, rel_to: usize) -> PhysicalUnit
        where BigInt: From<A> + From<R>
    {
        PhysicalUnit {
            name: name,
            abs: abs.into(),
            rel: Some((rel.into(), rel_to)),
        }
    }
}

/// An array type.
#[derive(Debug)]
pub struct ArrayType<'t> {
    /// The index subtypes.
    indices: Vec<&'t Type>,
    /// The element subtype.
    element: &'t Type,
}

impl<'t> Type for ArrayType<'t> {
    fn is_scalar(&self) -> bool { false }
    fn is_discrete(&self) -> bool { false }
    fn is_numeric(&self) -> bool { false }
    fn is_composite(&self) -> bool { true }
    fn as_any(&self) -> AnyType { AnyType::Array(self) }
}

impl<'t> Display for ArrayType<'t> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "array")?;
        Ok(())
    }
}

/// A null type.
///
/// This type is not strictly part of the VHDL type system. Rather, arrays that
/// have negative length degenerate into null arrays. We handle these types
/// explicitly, since they significantly change how types match.
///
/// # Example
///
/// ```
/// use moore_vhdl::ty2::{Type, NullType};
///
/// let ty = NullType;
///
/// assert_eq!(format!("{}", ty), "null");
/// assert_eq!(ty.is_scalar(), false);
/// assert_eq!(ty.is_discrete(), false);
/// assert_eq!(ty.is_numeric(), false);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct NullType;

impl Type for NullType {
    fn is_scalar(&self) -> bool { false }
    fn is_discrete(&self) -> bool { false }
    fn is_numeric(&self) -> bool { false }
    fn is_composite(&self) -> bool { false }
    fn as_any(&self) -> AnyType { AnyType::Null }
}

impl Display for NullType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "null")
    }
}

/// A universal integer.
///
/// This is not strictly a separate type, but rather defined by the standard as
/// the integer type with the largest range. However since we can represent
/// arbitrary numbers as `BigInt`, we use this special marker type.
///
/// # Example
///
/// ```
/// use moore_vhdl::ty2::{Type, UniversalIntegerType};
///
/// let ty = UniversalIntegerType;
///
/// assert_eq!(format!("{}", ty), "{universal integer}");
/// assert_eq!(ty.is_scalar(), true);
/// assert_eq!(ty.is_discrete(), true);
/// assert_eq!(ty.is_numeric(), true);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct UniversalIntegerType;

impl Type for UniversalIntegerType {
    fn is_scalar(&self) -> bool { true }
    fn is_discrete(&self) -> bool { true }
    fn is_numeric(&self) -> bool { true }
    fn is_composite(&self) -> bool { false }
    fn as_any(&self) -> AnyType { AnyType::UniversalInteger }
}

impl Display for UniversalIntegerType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{universal integer}}")
    }
}

/// A universal real.
///
/// This is not strictly a separate type, but rather defined by the standard as
/// the floating-point type with the largest range. We use this special marker
/// type.
///
/// # Example
///
/// ```
/// use moore_vhdl::ty2::{Type, UniversalRealType};
///
/// let ty = UniversalRealType;
///
/// assert_eq!(format!("{}", ty), "{universal real}");
/// assert_eq!(ty.is_scalar(), true);
/// assert_eq!(ty.is_discrete(), false);
/// assert_eq!(ty.is_numeric(), true);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct UniversalRealType;

impl Type for UniversalRealType {
    fn is_scalar(&self) -> bool { true }
    fn is_discrete(&self) -> bool { false }
    fn is_numeric(&self) -> bool { true }
    fn is_composite(&self) -> bool { false }
    fn as_any(&self) -> AnyType { AnyType::UniversalReal }
}

impl Display for UniversalRealType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{universal real}}")
    }
}