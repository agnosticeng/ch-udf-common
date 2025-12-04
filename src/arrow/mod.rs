use arrow::array::{ArrayRef, RecordBatch};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArrowExtError {
    #[error("column `{0}` not found")]
    ColumnNotFound(String),
    #[error("cannot downcast column `{0}`")]
    CannotDowncastColumn(String),
    #[error("cannot downcast array")]
    CannotDowncastArray,
}

pub trait RecordBatchExt {
    fn get_column<T: 'static>(&self, col_name: &str) -> Result<&T, ArrowExtError>;
}

impl RecordBatchExt for RecordBatch {
    fn get_column<T: 'static>(&self, col_name: &str) -> Result<&T, ArrowExtError> {
        let col = self
            .column_by_name(col_name)
            .ok_or(ArrowExtError::ColumnNotFound(col_name.to_string()))?;
        let arr = col
            .as_any()
            .downcast_ref()
            .ok_or(ArrowExtError::CannotDowncastColumn(col_name.to_string()));
        arr
    }
}

pub trait ArrayRefExt {
    fn as_array<T: 'static>(&self) -> Result<&T, ArrowExtError>;
}

impl ArrayRefExt for ArrayRef {
    fn as_array<T: 'static>(&self) -> Result<&T, ArrowExtError> {
        self.as_any()
            .downcast_ref()
            .ok_or(ArrowExtError::CannotDowncastArray)
    }
}
