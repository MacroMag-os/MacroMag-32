impl VosPipeline {
    pub fn execute(&self, mut data_stream: Vec<VosObject>) -> Vec<VosObject> {
        for transform in &self.transforms {
            match transform {
                VosTransform::Where { field, op, value } => {
                    data_stream.retain(|obj| {
                        obj.get(field).map_or(false, |v| Self::compare(v, op, value))
                    });
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

    fn compare(left: &VosValue, op: &CompareOp, right: &VosValue) -> bool {
        match (left, op, right) {
            (VosValue::Text(a), CompareOp::Eq, VosValue::Text(b)) => a == b,
            (VosValue::Bytes(a), CompareOp::Gt, VosValue::Bytes(b)) => a > b,
            (VosValue::Int(a), CompareOp::Gt, VosValue::Int(b)) => a > b,
            _ => false, // Tu można dopisać resztę operatorów
        }
    }
}
