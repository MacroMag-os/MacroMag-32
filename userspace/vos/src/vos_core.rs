use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum VosValue {
    Text(String),
    Int(i64),
    Float(f64),
    Bytes(u64),
    Bool(bool),
}

pub type VosObject = HashMap<String, VosValue>;

#[derive(Debug)]
pub struct VosPipeline {
    pub sandbox: Option<SandboxConfig>,
    pub source: VosCommand,
    pub transforms: Vec<VosTransform>,
}

#[derive(Debug)]
pub struct SandboxConfig {
    pub allow_net: bool,
    pub read_only_fs: bool,
}

#[derive(Debug)]
pub struct VosCommand {
    pub module: String,
    pub action: String,
    pub args: Vec<VosValue>,
}

#[derive(Debug, Clone)]
pub enum FilterExpr {
    Condition {
        field: String,
        op: CompareOp,
        value: VosValue,
    },
    And(Box<FilterExpr>, Box<FilterExpr>),
    Or(Box<FilterExpr>, Box<FilterExpr>),
}

#[derive(Debug)]
pub enum VosTransform {
    Where(FilterExpr),
    Select(Vec<String>),
    Limit(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompareOp {
    Eq, Neq, Gt, Lt, Gte, Lte,
}
