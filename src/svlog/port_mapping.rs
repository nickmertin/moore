// Copyright (c) 2016-2020 Fabian Schuiki

//! A port mapping generated by an instantiation.

use crate::{
    ast_map::AstNode,
    crate_prelude::*,
    hir::{NamedParam, PosParam},
    port_list::{ExtPort, PortedNode},
    ParamEnv,
};
use itertools::Itertools;
use std::{collections::HashSet, sync::Arc};

/// A port mapping.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct PortMapping<'a>(pub Vec<(Ref<'a, ExtPort<'a>>, NodeEnvId)>);

impl<'a> PortMapping<'a> {
    /// Find the signal assigned to a port.
    pub fn find(&self, node_id: NodeId) -> Option<NodeEnvId> {
        self.0
            .iter()
            .find(|&&(port, _)| port.id == node_id)
            .map(|&(_, id)| id)
    }

    /// Find the port a signal is assigned to.
    pub fn reverse_find(&self, node_id: NodeId) -> Option<&'a ExtPort<'a>> {
        self.0
            .iter()
            .find(|&&(_, id)| id.id() == node_id)
            .map(|&(Ref(port), _)| port)
    }
}

/// A location that implies a port mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PortMappingSource<'a> {
    ModuleInst {
        module: &'a dyn PortedNode<'a>,
        inst: NodeId,
        /// The parameter environment around the instantiation.
        outer_env: ParamEnv,
        /// The parameter environment generated by the instantiation.
        inner_env: ParamEnv,
        pos: &'a [PosParam],
        named: &'a [NamedParam],
    },
    InterfaceInst {
        interface: &'a dyn PortedNode<'a>,
        inst: NodeId,
        /// The parameter environment around the instantiation.
        outer_env: ParamEnv,
        /// The parameter environment generated by the instantiation.
        inner_env: ParamEnv,
        pos: &'a [PosParam],
        named: &'a [NamedParam],
    },
}

/// Compute the port assignments for an instantiation.
#[moore_derive::query]
pub(crate) fn port_mapping<'a>(
    cx: &impl Context<'a>,
    node: &'a dyn PortedNode<'a>,
    outer_env: ParamEnv,
    inner_env: ParamEnv,
    Ref(inst): Ref<'a, ast::InstName<'a>>,
    pos: &'a [hir::PosParam],
    named: &'a [hir::NamedParam],
    has_wildcard_port: bool,
) -> Result<Arc<PortMapping<'a>>> {
    trace!(
        "Compute port mapping for {:?} (outer {:?}, inner {:?}) at {:?}",
        node,
        outer_env,
        inner_env,
        inst
    );
    let port_list = cx.canonicalize_ports(node);

    // Associate the positional assignments with external ports.
    let pos_iter = pos.iter().enumerate().map(|(index, &(span, assign_id))| {
        match port_list.ext_pos.get(index) {
            Some(port) => Ok((port, assign_id)),
            None => {
                cx.emit(
                    DiagBuilder2::error(format!(
                        "{} only has {} ports(s)",
                        node,
                        port_list.ext_pos.len()
                    ))
                    .span(span),
                );
                Err(())
            }
        }
    });

    // Associate the named assignments with external ports.
    let named_iter = named.iter().map(|&(_span, name, assign_id)| {
        let names = match port_list.ext_named.as_ref() {
            Some(x) => x,
            None => {
                cx.emit(
                    DiagBuilder2::error(format!("{} requires positional connections", node))
                        .span(name.span)
                        .add_note(format!(
                            "The {:#} has unnamed ports which require connecting by position.",
                            node
                        ))
                        .add_note(format!("Remove `.{}(...)`", name)),
                );
                return Err(());
            }
        };
        match names.get(&name.value) {
            Some(&index) => Ok((&port_list.ext_pos[index], assign_id)),
            None => {
                cx.emit(
                    DiagBuilder2::error(format!("no port `{}` in {}", name, node,))
                        .span(name.span)
                        .add_note(format!(
                            "Declared ports are {}",
                            port_list
                                .ext_pos
                                .iter()
                                .flat_map(|n| n.name)
                                .map(|n| format!("`{}`", n))
                                .format(", ")
                        )),
                );
                Err(())
            }
        }
    });

    // Build a vector of ports.
    let ports: Result<Vec<_>> = pos_iter
        .chain(named_iter)
        .filter_map(|err| match err {
            Ok((port, Some(assign_id))) => Some(Ok((Ref(port), assign_id.env(outer_env)))),
            Ok(_) => None,
            Err(()) => Some(Err(())),
        })
        .collect();
    let mut ports = ports?;

    // If there was a wildcard connection present, resolve any ports not yet
    // connected.
    if has_wildcard_port {
        let handled: HashSet<_> = ports.iter().map(|(p, _)| *p).collect();
        let unhandled = port_list
            .ext_pos
            .iter()
            .filter(|p| !handled.contains(&Ref(p)));
        for port in unhandled {
            let name = match port.name {
                Some(n) => n,
                None => {
                    cx.emit(
                        DiagBuilder2::error(format!(
                            "unnamed ports in {} cannot be connected with wildcard `.*`",
                            node
                        ))
                        .span(inst.span),
                    );
                    return Err(());
                }
            };
            trace!("- Connecting port {:?} through wildcard", name);

            // Create a new helper AST identifier for this port.
            let expr = cx
                .arena()
                .alloc_ast_expr(ast::Expr::new(name.span, ast::IdentExpr(name)));
            expr.link_attach(inst, inst.order());
            let assign_id = cx.map_ast_with_parent(AstNode::Expr(expr), inst.id());
            ports.push((Ref(port), assign_id.env(outer_env)));
        }
    }

    Ok(Arc::new(PortMapping(ports)))
}
