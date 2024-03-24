use arrow::array::RecordBatchReader;

pub trait Handler {
    // trait object name
    fn get_name(&self) -> String;

    // trait object process record batch stream
    fn process(
        &self,
        input: Box<dyn RecordBatchReader + Send>,
    ) -> Box<dyn RecordBatchReader + Send>;
}

pub type DynHandler = Box<dyn Handler>;
