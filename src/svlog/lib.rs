// Copyright (c) 2016-2017 Fabian Schuiki

//! This crate implements SystemVerilog for the moore compiler.

extern crate moore_common;
extern crate bincode;
extern crate rustc_serialize;

pub mod ast;
pub mod cat;
pub mod lexer;
pub mod parser;
pub mod preproc;
pub mod store;
pub mod token;
pub mod resolve;
pub mod renumber;
pub mod hir;