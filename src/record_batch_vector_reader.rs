use arrow::array::RecordBatchReader;
use arrow::record_batch::RecordBatch;
use arrow_schema::{ArrowError, SchemaRef};

pub struct RecordBatchVectorReader {
    batches: Vec<RecordBatch>,
    schema: SchemaRef,
    position: usize,
}

impl RecordBatchVectorReader {
    pub fn new(schema: SchemaRef, batches: Vec<RecordBatch>) -> Box<RecordBatchVectorReader> {
        Box::new(RecordBatchVectorReader {
            batches,
            schema,
            position: 0,
        })
    }
}

impl Iterator for RecordBatchVectorReader {
    type Item = Result<RecordBatch, ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.batches.len() {
            self.position += 1;
            Some(Ok(self.batches[self.position - 1].clone()))
        } else {
            None
        }
    }
}

impl RecordBatchReader for RecordBatchVectorReader {
    fn schema(&self) -> SchemaRef {
        return self.schema.clone();
    }
}

#[cfg(test)]
mod tests {
    use arrow::array::{Array, Int64Array};
    use arrow::datatypes::DataType::Int64;
    use arrow::record_batch::RecordBatch;
    use arrow_schema::{Field, Schema};
    use std::sync::Arc;

    use super::*;

    #[test]
    fn test_no_op_handler() {
        let array1 = Arc::new(Int64Array::from(vec![Some(8), Some(7), Some(1), Some(2)]));
        let array2 = Arc::new(Int64Array::from(vec![Some(9), Some(3), Some(2), None]));
        let arrays: Vec<Arc<dyn Array>> = vec![array1, array2];
        let schema = Arc::new(Schema::new(vec![
            Field::new("a", Int64, true),
            Field::new("b", Int64, true),
        ]));
        let batch = RecordBatch::try_new(schema.clone(), arrays).unwrap();
        let mut reader: Box<dyn RecordBatchReader + Send> =
            RecordBatchVectorReader::new(schema.clone(), vec![batch.clone(), batch.clone()]);
        assert_eq!(reader.schema(), schema);
        let next = reader.next();
        assert!(next.is_some());
        let next_batch = next.unwrap().unwrap();
        assert_eq!(next_batch, batch);
        assert!(reader.next().is_some());
        assert!(reader.next().is_none());
    }
}
