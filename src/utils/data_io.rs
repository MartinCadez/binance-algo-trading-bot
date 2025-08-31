use polars::prelude::*;
use std::path::Path;

pub fn read_parquet(path: &str) -> PolarsResult<DataFrame> {
    let abs_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(path)
        .canonicalize()?;

    let file = std::fs::File::open(abs_path)?;
    ParquetReader::new(file).finish()
}
