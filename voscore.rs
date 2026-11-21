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
    Where(FilterExpr), // Zamiast pojedynczego warunku trzymamy całe drzewo logiczne
    Select(Vec<String>),
    Limit(usize),
}
