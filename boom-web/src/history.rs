use std::{fs::File, sync::Arc, time::Duration};

use boom_config::get_default_config_path;
use tokio::time::{Instant, interval_at};
use tracing::info;

use std::{fs::OpenOptions, io};

use boom_core::{HistoryEntry, cache::SEARCH_HISTORY_CACHE};
use parquet::{
    basic::BrotliLevel,
    data_type::{ByteArray, ByteArrayType, Int64Type},
    file::{
        properties::WriterPropertiesBuilder,
        writer::{SerializedFileWriter, SerializedRowGroupWriter},
    },
    schema::parser::parse_message_type,
};

const SCHEMA_STR: &str = "message schema {
        REQUIRED BINARY bang (UTF8);
        REQUIRED BINARY query (UTF8);
        REQUIRED INT64 timestamp;
    }";

pub async fn save_history(interval: Duration) {
    let mut history_save_interval = interval_at(Instant::now() + interval, interval);
    loop {
        history_save_interval.tick().await;
        info!("Updating search history save");

        let rlock = SEARCH_HISTORY_CACHE
            .read()
            .expect("Search History Cache should be readable");
        if rlock.is_empty() {
            continue;
        }

        let hist_file_path = get_default_config_path()
            .parent()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No parent directory"))
            .unwrap()
            .join("hist_file.parquet");
        let hist_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&hist_file_path)
            .expect("History file should be writeable");
        let mut writer = SerializedFileWriter::new(
            hist_file,
            Arc::new(parse_message_type(SCHEMA_STR).unwrap()),
            WriterPropertiesBuilder::default()
                .set_compression(parquet::basic::Compression::BROTLI(
                    BrotliLevel::try_new(4).unwrap(),
                ))
                .build()
                .into(),
        )
        .unwrap();

        let mut row_group_writer = writer.next_row_group().unwrap();

        write_column::<_, ByteArray, ByteArrayType>(&rlock, &mut row_group_writer, |hist| {
            ByteArray::from(hist.query.0.as_str())
        })
        .and_then(|()| {
            write_column::<_, ByteArray, ByteArrayType>(&rlock, &mut row_group_writer, |hist| {
                ByteArray::from(hist.query.1.as_str())
            })
        })
        .and_then(|()| {
            write_column::<_, _, Int64Type>(&rlock, &mut row_group_writer, |hist| hist.timestamp)
        })
        .expect("Could not write history entry into column");

        row_group_writer.close().unwrap();
        writer.close().unwrap();

        drop(rlock);
    }
}

fn write_column<T, S, W>(
    history: &[HistoryEntry],
    row_group_writer: &mut SerializedRowGroupWriter<'_, File>,
    map: T,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: FnMut(&HistoryEntry) -> S,
    W: parquet::data_type::DataType<T = S>,
{
    if let Some(mut col_writer) = row_group_writer.next_column()? {
        let values: Vec<_> = history.iter().map(map).collect();
        let typed_writer = col_writer.typed::<W>();
        typed_writer.write_batch(&values, None, None)?;
        col_writer.close()?;
    }
    Ok(())
}
