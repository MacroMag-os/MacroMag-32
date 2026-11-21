use std::collections::HashMap;

/// Reprezentacja typowanych wartości w potokach VOS
#[derive(Debug, Clone, PartialEq)]
pub enum VosValue {
    Text(String),
    Int(i64),
    Float(f64),
    Bytes(u64), // Np. 512MB przeliczane od razu na bajty
    Bool(bool),
}

/// Pojedynczy obiekt przepływający przez potok (np. proces, plik, maszyna VM)
pub type VosObject = HashMap<String, VosValue>;

/// Węzły drzewa składniowego (AST) dla parsera VOS
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
    pub module: String, // np. "vm" lub "sys"
    pub action: String, // np. "list" lub "ps"
    pub args: Vec<VosValue>,
}

#[derive(Debug)]
pub enum VosTransform {
    Where {
        field: String,
        op: CompareOp,
        value: VosValue,
    },
    Select(Vec<String>),
    Limit(usize),
}

#[derive(Debug)]
pub enum CompareOp {
    Eq, Neq, Gt, Lt, Gte, Lte,
}
