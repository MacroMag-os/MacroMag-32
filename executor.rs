use crate::vos_core::{CompareOp, FilterExpr, VosObject, VosPipeline, VosTransform, VosValue};

impl VosPipeline {
    pub fn execute(&self, mut data_stream: Vec<VosObject>) -> Vec<VosObject> {
        for transform in &self.transforms {
            match transform {
                VosTransform::Where(expr) => {
                    data_stream.retain(|obj| Self::eval_filter(obj, expr));
                }
                VosTransform::Select(fields) => {
                    for obj in &mut data_stream {
                        obj.retain(|k, _| fields.contains(k));
                    }
                }
                VosTransform::Limit(n) => {
                    data_stream.truncate(*n);
                }
            }
        }
        data_stream
    }

    fn eval_filter(obj: &VosObject, expr: &FilterExpr) -> bool {
        match expr {
            FilterExpr::Condition { field, op, value } => {
                obj.get(field).map_or(false, |v| Self::compare(v, op, value))
            }
            // Leniwa ewaluacja (short-circuit): jeśli lewa strona jest false, prawa nie jest sprawdzana
            FilterExpr::And(left, right) => {
                Self::eval_filter(obj, left) && Self::eval_filter(obj, right)
            }
            FilterExpr::Or(left, right) => {
                Self::eval_filter(obj, left) || Self::eval_filter(obj, right)
            }
        }
    }

    fn compare(left: &VosValue, op: &CompareOp, right: &VosValue) -> bool {
        match (left, right) {
            (VosValue::Text(a), VosValue::Text(b)) => match op {
                CompareOp::Eq => a == b,
                CompareOp::Neq => a != b,
                _ => false,
            },
            (VosValue::Bytes(a), VosValue::Bytes(b)) => Self::cmp_ord(a, op, b),
            (VosValue::Int(a), VosValue::Int(b)) => Self::cmp_ord(a, op, b),
            (VosValue::Bool(a), VosValue::Bool(b)) => match op {
                CompareOp::Eq => a == b,
                CompareOp::Neq => a != b,
                _ => false,
            },
            _ => false,
        }
    }

    fn cmp_ord<T: Ord>(a: &T, op: &CompareOp, b: &T) -> bool {
        match op {
            CompareOp::Eq => a == b,
            CompareOp::Neq => a != b,
            CompareOp::Gt => a > b,
            CompareOp::Lt => a < b,
            CompareOp::Gte => a >= b,
            CompareOp::Lte => a <= b,
        }
    }
}
