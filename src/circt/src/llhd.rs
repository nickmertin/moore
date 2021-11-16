// Copyright (c) 2016-2021 Fabian Schuiki

use crate::crate_prelude::*;

pub fn dialect() -> DialectHandle {
    DialectHandle::from_raw(unsafe { circt_sys::mlirGetDialectHandle__llhd__() })
}

/// Create a new time type.
pub fn get_time_type(cx: Context) -> Type {
    Type::from_raw(unsafe { llhdTimeTypeGet(cx.raw()) })
}

/// Create a new signal type.
pub fn get_signal_type(element: Type) -> Type {
    Type::from_raw(unsafe { llhdSignalTypeGet(element.raw()) })
}

/// Create a new pointer type.
pub fn get_pointer_type(element: Type) -> Type {
    Type::from_raw(unsafe { llhdSignalTypeGet(element.raw()) })
}

/// Get the element type of signal type.
pub fn signal_type_element(ty: Type) -> Type {
    Type::from_raw(unsafe { llhdSignalTypeGetElementType(ty.raw()) })
}

/// Get the element type of pointer type.
pub fn pointer_type_element(ty: Type) -> Type {
    Type::from_raw(unsafe { llhdPointerTypeGetElementType(ty.raw()) })
}

/// Create a new integer attribute.
pub fn get_time_attr(
    cx: Context,
    seconds: &BigRational,
    delta: usize,
    epsilon: usize,
) -> Attribute {
    // TODO: This is super hacky. We need a better way to capture the arbitrary
    // time granularity.
    let ps = (seconds * BigInt::from(10).pow(12)).to_u64().unwrap();
    Attribute::from_raw(unsafe {
        llhdTimeAttrGet(
            cx.raw(),
            mlirStringRefCreateFromStr("ps"),
            ps,
            delta as _,
            epsilon as _,
        )
    })
}

def_operation!(EntityOp, "llhd.entity");
def_operation!(ProcessOp, "llhd.proc");

pub trait EntityLike: SingleBlockOp {
    /// Get the number of ports.
    fn num_ports(&self) -> usize {
        unsafe { mlirBlockGetNumArguments(self.block()) as usize }
    }

    /// Get the number of input ports.
    fn num_inputs(&self) -> usize {
        self.attr_usize("ins")
    }

    /// Get the number of output ports.
    fn num_outputs(&self) -> usize {
        self.num_ports() - self.num_inputs()
    }

    /// Get a port by index.
    fn port(&self, index: usize) -> Value {
        unsafe { Value::from_raw(mlirBlockGetArgument(self.block(), index as _)) }
    }

    /// Get an input port by index.
    fn input(&self, index: usize) -> Value {
        assert!(index < self.num_inputs());
        self.port(index)
    }

    /// Get an input port by index.
    fn output(&self, index: usize) -> Value {
        assert!(index < self.num_outputs());
        self.port(index + self.num_inputs())
    }

    /// Get an iterator over all ports.
    fn ports(&self) -> Box<dyn Iterator<Item = Value> + '_> {
        Box::new((0..self.num_ports()).map(move |i| self.port(i)))
    }

    /// Get an iterator over the input ports.
    fn input_ports(&self) -> Box<dyn Iterator<Item = Value> + '_> {
        Box::new((0..self.num_inputs()).map(move |i| self.input(i)))
    }

    /// Get an iterator over the output ports.
    fn output_ports(&self) -> Box<dyn Iterator<Item = Value> + '_> {
        Box::new((0..self.num_outputs()).map(move |i| self.output(i)))
    }
}

impl SingleRegionOp for EntityOp {}
impl SingleBlockOp for EntityOp {}
impl EntityLike for EntityOp {}

impl SingleRegionOp for ProcessOp {}
impl SingleBlockOp for ProcessOp {}
impl EntityLike for ProcessOp {}

pub struct EntityLikeBuilder<'a> {
    name: &'a str,
    inputs: Vec<(&'a str, Type)>,
    outputs: Vec<(&'a str, Type)>,
}

impl<'a> EntityLikeBuilder<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            inputs: vec![],
            outputs: vec![],
        }
    }

    /// Add an input port.
    pub fn add_input(&mut self, name: &'a str, ty: Type) -> &mut Self {
        self.inputs.push((name, ty));
        self
    }

    /// Add an output port.
    pub fn add_output(&mut self, name: &'a str, ty: Type) -> &mut Self {
        self.outputs.push((name, ty));
        self
    }

    /// Build an entity.
    pub fn build_entity(&mut self, builder: &mut Builder) -> EntityOp {
        self.build(builder)
    }

    /// Build a process.
    pub fn build_process(&mut self, builder: &mut Builder) -> ProcessOp {
        self.build(builder)
    }

    fn build<Op: EntityLike>(&mut self, builder: &mut Builder) -> Op {
        builder.build_with(|builder, state| {
            let types = self
                .inputs
                .iter()
                .chain(self.outputs.iter())
                .map(|(_, ty)| *ty);
            let mlir_types: Vec<MlirType> = types.clone().map(|x| x.raw()).collect();

            state.add_attribute("sym_name", get_string_attr(builder.cx, self.name));
            state.add_attribute(
                "type",
                get_type_attr(get_function_type(builder.cx, types, None)),
            );
            state.add_attribute(
                "ins",
                get_integer_attr(get_integer_type(builder.cx, 64), self.inputs.len() as _),
            );

            unsafe {
                let region = mlirRegionCreate();
                mlirRegionAppendOwnedBlock(
                    region,
                    mlirBlockCreate(mlir_types.len() as _, mlir_types.as_ptr()),
                );
                mlirOperationStateAddOwnedRegions(state.raw_mut(), 1, [region].as_ptr());
            }
        })
    }
}

def_operation_single_result!(ConstantTimeOp, "llhd.constant_time");
def_operation_single_result!(SignalOp, "llhd.sig");
def_operation_single_result!(VariableOp, "llhd.var");
def_operation!(ConnectOp, "llhd.con");
def_operation_single_result!(ProbeOp, "llhd.prb");
def_operation!(DriveOp, "llhd.drv");
def_operation_single_result!(LoadOp, "llhd.ld");
def_operation!(StoreOp, "llhd.st");

impl ConstantTimeOp {
    /// Create a new constant time value.
    pub fn new(builder: &mut Builder, seconds: &BigRational, delta: usize, epsilon: usize) -> Self {
        builder.build_with(|builder, state| {
            state.add_attribute("value", get_time_attr(builder.cx, seconds, delta, epsilon));
            state.add_result(get_time_type(builder.cx));
        })
    }

    /// Create a new seconds time value.
    pub fn with_seconds(builder: &mut Builder, seconds: &BigRational) -> Self {
        Self::new(builder, seconds, 0, 0)
    }

    /// Create a new delta time value.
    pub fn with_delta(builder: &mut Builder, delta: usize) -> Self {
        Self::new(builder, &BigRational::zero(), delta, 0)
    }

    /// Create a new epsilon time value.
    pub fn with_epsilon(builder: &mut Builder, epsilon: usize) -> Self {
        Self::new(builder, &BigRational::zero(), 0, epsilon)
    }
}

impl SignalOp {
    /// Create a new signal.
    pub fn new(builder: &mut Builder, name: &str, init: Value) -> Self {
        builder.build_with(|builder, state| {
            state.add_operand(init);
            state.add_attribute("name", get_string_attr(builder.cx, name));
            state.add_result(get_signal_type(init.ty()));
        })
    }
}

impl VariableOp {
    /// Create a new variable.
    pub fn new(builder: &mut Builder, init: Value) -> Self {
        builder.build_with(|_, state| {
            state.add_operand(init);
            state.add_result(get_signal_type(init.ty()));
        })
    }
}

impl ConnectOp {
    /// Create a new signal connection.
    pub fn new(builder: &mut Builder, sig1: Value, sig2: Value) -> Self {
        builder.build_with(|_, state| {
            state.add_operand(sig1);
            state.add_operand(sig2);
        })
    }
}

impl ProbeOp {
    /// Probe a signal.
    pub fn new(builder: &mut Builder, sig: Value) -> Self {
        builder.build_with(|_, state| {
            let ty = signal_type_element(sig.ty());
            state.add_operand(sig);
            state.add_result(ty);
        })
    }
}

impl DriveOp {
    /// Drive a value onto a signal.
    pub fn new(builder: &mut Builder, sig: Value, value: Value, delay: Value) -> Self {
        builder.build_with(|_, state| {
            state.add_operand(sig);
            state.add_operand(value);
            state.add_operand(delay);
        })
    }
}

impl LoadOp {
    /// Load the value of a variable.
    pub fn new(builder: &mut Builder, var: Value) -> Self {
        builder.build_with(|_, state| {
            let ty = pointer_type_element(var.ty());
            state.add_operand(var);
            state.add_result(ty);
        })
    }
}

impl StoreOp {
    /// Store a value to a variable.
    pub fn new(builder: &mut Builder, var: Value, value: Value) -> Self {
        builder.build_with(|_, state| {
            state.add_operand(var);
            state.add_operand(value);
        })
    }
}