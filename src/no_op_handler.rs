use crate::handler::Handler;
use crate::record_batch_vector_reader::RecordBatchVectorReader;
use arrow::array::RecordBatchReader;
use arrow::record_batch::RecordBatch;

pub struct NoOpHandler {}

impl NoOpHandler {
    pub fn new() -> Self {
        NoOpHandler {}
    }
}

impl Handler for NoOpHandler {
    fn get_name(&self) -> String {
        "NoOp".to_string()
    }

    fn process(
        &self,
        mut input: Box<dyn RecordBatchReader + Send>,
    ) -> Box<dyn RecordBatchReader + Send> {
        let mut ret_batches: Vec<RecordBatch> = Vec::new();
        let schema = input.schema();
        loop {
            if let Some(batch_result) = input.next() {
                if let Ok(batch) = batch_result {
                    ret_batches.push(batch.clone());
                }
            } else {
                break;
            }
        }
        return RecordBatchVectorReader::new(schema, ret_batches);
    }
}

pub type DynHandler = Box<dyn Handler>;

#[cfg(test)]
mod tests {
    use arrow::array::{Array, Int32Array};
    use arrow::datatypes::DataType::Int32;
    use arrow::record_batch::RecordBatch;
    use arrow_schema::{Field, Schema};
    use std::sync::Arc;

    use super::*;

    #[test]
    fn test_no_op_handler() {
        let array = Arc::new(Int32Array::from(vec![Some(2), None, Some(1), None]));
        let arrays: Vec<Arc<dyn Array>> = vec![array.clone(), array.clone(), array.clone()];
        let schema = Arc::new(Schema::new(vec![
            Field::new("a", Int32, true),
            Field::new("b", Int32, true),
            Field::new("c", Int32, true),
        ]));
        let batch = RecordBatch::try_new(schema.clone(), arrays).unwrap();
        let reader: Box<dyn RecordBatchReader + Send> =
            RecordBatchVectorReader::new(schema, vec![batch.clone(), batch.clone()]);

        let handler = NoOpHandler {};
        let mut result_reader = handler.process(reader);
        let result_batch = result_reader.next().unwrap().unwrap();
        assert_eq!(result_batch, batch);
    }
}
