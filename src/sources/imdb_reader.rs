use polars::prelude::*;

/// currently unimpl'd: will be used to download IMDB dataset on init
pub fn download_source() {
    unimplemented!();
}

pub fn tv_show_ids() -> DataFrame {
    let fname = "./title.basics.tsv";

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
        .sort("startYear", Default::default())
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

    let df = q.collect().unwrap();

    println!("{}", df);

    df
}
