use arrow::array::{Array, BinaryArray};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use std::fs::File;
use std::path::PathBuf;

use std::collections::HashSet;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use xorfilter::BuildHasherDefault;
use xorfilter::Xor8Builder;

use bloomfilter::Bloom;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = PathBuf::from("/data1/b41sh/tt5/amazon_reviews_2010.snappy.parquet");

    let file = File::open(&file_path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;

    let schema = builder.schema();
    println!("Schema: {:?}", schema);

    let mut reader = builder.build()?;

    let mut i = 0;
    let mut values = Vec::new();
    while let Some(record_batch) = reader.next() {
        match record_batch {
            Ok(batch) => {
                println!("i={:?}", i);
                let array = batch.column_by_name("review_body").unwrap();

                let binary_array = array.as_any().downcast_ref::<BinaryArray>().unwrap();
                //let mut block_values = Vec::with_capacity(binary_array.len());
                for i in 0..binary_array.len() {
                    if !binary_array.is_null(i) {
                        let value = binary_array.value(i);
                        values.push(unsafe { String::from_utf8_unchecked(value.to_vec()) });
                        //block_values.push(unsafe { String::from_utf8_unchecked(value.to_vec()) });
                    }
                }
                //values.push(block_values);
            }
            Err(e) => {
                eprintln!("Error reading record batch: {}", e);
            }
        }
        i += 1;
        if i == 100 {
            break;
        }
    }

    let block_values = split_values(values, 10000);
    println!("block_num={:?}", block_values.len());
    //let gram_size = 10;
    //let bitmap_size = 128 * 1024 * 1024;
    //let bitmap_size = 10 * 1024 * 1024;
    //let bitmap_size = 1024 * 1024;
    //let bitmap_size = 60 * 1024;

    for gram_size in 3..10 {
        let bitmap_sizes = vec![
            100 * 1024,
            500 * 1024,
            1024 * 1024,
            5 * 1024 * 1024,
        ];
        for bitmap_size in bitmap_sizes {
            test_bloom(gram_size, bitmap_size, &block_values);
        }
    }

    Ok(())
}

fn test_bloom(gram_size: usize, bitmap_size: usize, values: &Vec<Vec<String>>) {
    let search_keys = vec![
        "Frame and discovered".to_string(),
        "I have not used".to_string(),
        "I cook my oats in a crockpot overnight.".to_string(),
        "It would have taken a lot of work as the author".to_string(),
        "brief history of the life of George Washington.".to_string(),
        "aaaaaaaa bbbbbbbb not exist".to_string(),
        "The first track with Chris Botti is beautiful".to_string(),
        "is beautif".to_string(),
        "s beautifu".to_string(),
        "wwwwwwwwww".to_string(),
    ];

    let mut search_key_hashs = Vec::with_capacity(search_keys.len());
    let mut search_key_no_hashs = Vec::with_capacity(search_keys.len());
    for search_key in &search_keys {
        let mut search_key_hash = HashSet::new();
        let mut search_key_no_hash = HashSet::new();
        for i in 0..search_key.len() - gram_size {
            let key = &search_key[i..i + gram_size];
            search_key_no_hash.insert(key.to_string());
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            search_key_hash.insert(hasher.finish());
        }
        search_key_hashs.push(search_key_hash);
        search_key_no_hashs.push(search_key_no_hash);
    }

    let mut bloom_failed = 0;
    let mut xor8_failed = 0;
    let mut all_count = 0;

    for (i, block_values) in values.iter().enumerate() {
        //println!("block_values.len={:?}", block_values.len());
        let mut mem_size = 0;
        for text in block_values {
            mem_size += text.len();
        }
        //println!("mem_size={:?}", mem_size);

        let mut set = HashSet::new();
        let mut real_matched = HashSet::new();
        let mut no_hash_token = HashSet::new();
        for text in block_values {
            for (k, search_key) in search_keys.iter().enumerate() {
                if text.contains(search_key) {
                    real_matched.insert(k);
                }
            }

            let indices: Vec<_> = text.char_indices().map(|(i, _)| i).collect();
            let char_count = indices.len();
            if char_count < gram_size {
                continue;
            }
            let times = char_count - gram_size + 1;
            for j in 0..times {
                let start = indices[j];
                let end = if j + gram_size < char_count {
                    indices[j + gram_size]
                } else {
                    text.len()
                };
                let key = text[start..end].to_string();
                no_hash_token.insert(key.to_string());

                let mut hasher = DefaultHasher::new();
                key.hash(&mut hasher);
                set.insert(hasher.finish());
            }
        }
        //println!("set len={:?}", set.len());

        // build xor8
        let mut new_keys = Vec::new();
        for key in &set {
            //let new_key = key % bitmap_size as u64;
            let new_key = key;
            new_keys.push(new_key);
        }

        let hash_builder = BuildHasherDefault::default();
        let mut builder = Xor8Builder::with_hasher(hash_builder);
        builder.populate(&new_keys);
        let xor8_filter = builder.build().expect("build failed");

        // build bloom
        let mut bloom = Bloom::new(bitmap_size, set.len()).unwrap();
        for hash in set {
            bloom.set(&hash);
        }

        for ((k, search_key), search_key_hash_vec) in
            search_keys.iter().enumerate().zip(&search_key_hashs)
        {
            //println!("\nsearch_key={:?}", search_key);
            let is_real = real_matched.contains(&k);
            //if is_real {
            //    println!("block {} is real matched", i);
            //} else {
            //    println!("block {} is real not matched", i);
            //}

            let mut bloom_matched_num = 0;
            for search_key_hash in search_key_hash_vec {
                if bloom.check(&search_key_hash) {
                    bloom_matched_num += 1;
                }
            }
            let mut is_bloom = false;
            if bloom_matched_num == search_key_hash_vec.len() {
                is_bloom = true;
                //println!("block {} bloom is matched", i);
            //} else {
                //println!("block {} bloom is not matched", i);
            }

            let mut xor8_matched_num = 0;
            for search_key_hash in search_key_hash_vec {
                //let new_hash = search_key_hash % bitmap_size as u64;
                let new_hash = search_key_hash;
                if xor8_filter.contains(&new_hash) {
                    xor8_matched_num += 1;
                }
            }
            let mut is_xor8 = false;
            if xor8_matched_num == search_key_hash_vec.len() {
                is_xor8 = true;
                //println!("block {} xor8 is matched", i);
            //} else {
                //println!("block {} xor8 is not matched", i);
            }

            if is_real != is_bloom {
                bloom_failed += 1;
            }
            if is_real != is_xor8 {
                xor8_failed += 1;
            }
            all_count += 1;
        }

        //println!("\n");

        //let bytes = bloom.to_bytes();
        //println!("block {} bloom bytes len={:?}", i, bytes.len());

        //let bytes = xor8_filter.to_bytes();
        //println!("block {} xor8 bytes len={:?}", i, bytes.len());
    }

    let bloom_failed_ratio = bloom_failed as f64 / all_count as f64;
    let xor8_failed_ratio = xor8_failed as f64 / all_count as f64;

    println!("ngram={:?} bf_size={:?} bloom_failed={:?} xor_failed={:?}", gram_size, bitmap_size, bloom_failed_ratio, xor8_failed_ratio);
    //println!("\nbloom_failed_ratio={:?}", bloom_failed_ratio);
    //println!("xor8_failed_ratio={:?}", xor8_failed_ratio);
}


fn split_values(input: Vec<String>, chunk_size: usize) -> Vec<Vec<String>> {
    let mut result: Vec<Vec<String>> = Vec::new();
    let mut current_chunk: Vec<String> = Vec::new();

    for s in input {
        current_chunk.push(s);
        if current_chunk.len() == chunk_size {
            result.push(current_chunk);
            current_chunk = Vec::new();
        }
    }

    if !current_chunk.is_empty() {
        result.push(current_chunk);
    }

    result
}

