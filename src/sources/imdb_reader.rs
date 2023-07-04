use polars::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::models::TraktShow;

/// currently unimpl'd: will be used to download IMDB dataset on init
pub fn download_source() {
    unimplemented!();
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImdbShow {
    pub tconst: String,
    pub primary_title: Option<String>,
    pub original_title: Option<String>,
    pub start_year: Option<i64>,
    pub end_year: Option<i64>,
}

pub fn tv_show_ids() -> DataFrame {
    let fname = "./title.basics.short.tsv";

    let mut schema = Schema::new();
    schema.with_column("endYear".to_string().into(), DataType::Int64);

    let q = LazyCsvReader::new(fname)
        .has_header(true)
        .with_delimiter("\t".as_bytes()[0])
        .with_ignore_errors(true)
        .with_null_values(Some(NullValues::AllColumnsSingle(String::from("\\N"))))
        .with_dtype_overwrite(Some(&schema))
        .finish()
        .unwrap()
        .sort(
            "startYear",
            SortOptions {
                descending: true,
                nulls_last: true,
                multithreaded: true,
            },
        )
        .filter(
            // you should be able to do is_in here but i couldn't figure out the syntax?
            col("titleType")
                .eq(lit("tvSeries"))
                .or(col("titleType").eq(lit("tvMiniSeries"))),
        )
        .select(&[
            col("tconst"),
            col("primaryTitle"),
            col("originalTitle"),
            col("startYear"),
            col("endYear"),
        ]);

    q.with_streaming(true).collect().unwrap()
}

pub fn get_show_vec() -> Vec<TraktShow> {
    let df = tv_show_ids().head(Some(100));

    let fields = df.get_columns();
    let columns: Vec<&str> = fields.iter().map(|x| x.name()).collect();

    let mut items: Vec<TraktShow> = vec![];

    for idx in 0..df.height() {
        let mut map = std::collections::HashMap::new();

        let row = df.get_row(idx).unwrap();

        for (column, elem) in std::iter::zip(&columns, &mut row.0.iter()) {
            let value = match elem {
                AnyValue::Null => json!(Option::<String>::None),
                AnyValue::Utf8(val) => json!(val),
                AnyValue::Int64(val) => json!(val),
                other => unimplemented!("{:?}", other),
            };
            map.insert(*column as &str, value);
        }

        let j_text = serde_json::to_string(&map).unwrap();
        let j_row = serde_json::from_str::<ImdbShow>(&j_text).unwrap();

        items.push(TraktShow {
            imdb_id: j_row.tconst,
            trakt_id: None,
            tmdb_id: None,
            primary_title: j_row.primary_title.unwrap(),
            original_title: j_row.original_title.unwrap(),
            release_year: Some(j_row.start_year.unwrap() as i32),
            no_seasons: None,
            no_episodes: None,
            country: None,
            network: None,
        });
    }

    items
}