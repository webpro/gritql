use std::collections::BTreeMap;

use super::{
    compiler::CompilationContext,
    patterns::{Matcher, Name, Pattern},
    resolved_pattern::ResolvedPattern,
    state::State,
    variable::VariableSourceLocations,
    Context,
};
use crate::binding::Constant;
use anyhow::{anyhow, Result};
use marzano_util::analysis_logs::AnalysisLogs;
use tree_sitter::Node;

#[derive(Debug, Clone)]
pub struct Divide {
    pub(crate) lhs: Pattern,
    pub(crate) rhs: Pattern,
}

impl Divide {
    pub fn new(lhs: Pattern, rhs: Pattern) -> Self {
        Self { lhs, rhs }
    }

    pub(crate) fn from_node(
        node: &Node,
        context: &CompilationContext,
        vars: &mut BTreeMap<String, usize>,
        vars_array: &mut Vec<Vec<VariableSourceLocations>>,
        scope_index: usize,
        global_vars: &mut BTreeMap<String, usize>,
        logs: &mut AnalysisLogs,
    ) -> Result<Self> {
        let left = node
            .child_by_field_name("left")
            .ok_or_else(|| anyhow!("missing left of divide"))?;
        let left = Pattern::from_node(
            &left,
            context,
            vars,
            vars_array,
            scope_index,
            global_vars,
            false,
            logs,
        )?;

        let right = node
            .child_by_field_name("right")
            .ok_or_else(|| anyhow!("missing right of divide"))?;
        let right = Pattern::from_node(
            &right,
            context,
            vars,
            vars_array,
            scope_index,
            global_vars,
            false,
            logs,
        )?;

        Ok(Self::new(left, right))
    }

    pub(crate) fn call<'a>(
        &'a self,
        state: &mut State<'a>,
        context: &Context<'a>,
        logs: &mut AnalysisLogs,
    ) -> Result<ResolvedPattern<'a>> {
        let res = self.evaluate(state, context, logs)?;
        Ok(ResolvedPattern::Constant(Constant::Float(res)))
    }

    fn evaluate<'a>(
        &'a self,
        state: &mut State<'a>,
        context: &Context<'a>,
        logs: &mut AnalysisLogs,
    ) -> Result<f64> {
        let lhs = self.lhs.float(state, context, logs)?;
        let rhs = self.rhs.float(state, context, logs)?;
        let res = lhs / rhs;
        Ok(res)
    }
}

impl Name for Divide {
    fn name(&self) -> &'static str {
        "DIVIDE"
    }
}

impl Matcher for Divide {
    fn execute<'a>(
        &'a self,
        binding: &ResolvedPattern<'a>,
        state: &mut State<'a>,
        context: &Context<'a>,
        logs: &mut AnalysisLogs,
    ) -> Result<bool> {
        let binding_text = binding.text(&state.files)?;
        let binding_int = binding_text.parse::<f64>()?;
        let target = self.evaluate(state, context, logs)?;
        Ok(binding_int == target)
    }
}
