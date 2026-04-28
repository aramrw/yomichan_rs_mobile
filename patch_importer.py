import sys

with open('../../yomichan_rs_workspace/yomichan_rs/src/database/dictionary_importer.rs', 'r') as f:
    content = f.read()

# 1. Update CHUNKS to 999
content = content.replace("pub const CHUNKS: usize = 90;", "pub const CHUNKS: usize = 999;")

# 2. Fix the memory spike
old_block = """    tracing::info!("Inserting {} terms...", external_data.term_list.len());
    let serialized_terms: Vec<SerializedTerm> = external_data
        .term_list
        .into_par_iter()
        .map(|t| {
            let entry = DatabaseTermEntry {
                id: t.0.clone(),
                expression: t.1.clone(),
                reading: t.2.clone(),
                expression_reverse: t.3.clone(),
                reading_reverse: t.4.clone(),
                definition_tags: t.5.map(|s| s.to_string()),
                tags: t.6.map(|s| s.to_string()),
                rules: t.7.to_string(),
                score: t.8,
                glossary: t
                    .9
                    .into_iter()
                    .map(|g| match g {
                        yomichan_importer::structured_content::TermGlossaryGroupType::Content(c) => {
                            TermGlossaryGroupType::Content(TermGlossaryContentGroup {
                                plain_text: c.plain_text,
                                html: c.html,
                            })
                        }
                        yomichan_importer::structured_content::TermGlossaryGroupType::Deinflection(d) => {
                            TermGlossaryGroupType::Deinflection(TermGlossaryDeinflection {
                                form_of: d.form_of,
                                rules: d.rules.iter().map(|s| s.to_owned()).collect(),
                            })
                        }
                    })
                    .collect(),
                sequence: t.10,
                term_tags: t.11.as_ref().map(|s| s.to_string()),
                dictionary: t.12.clone(),
                file_path: t.13.clone(),
            };
            // DO NOT UNCOMMENT. ITS MASSIVE
            //eprintln!("DEBUG: Serializing entry: {:?}", entry);
            let data_blob = encode(&entry).expect("Failed to encode");
            SerializedTerm {
                id: entry.id,
                expression: entry.expression,
                reading: entry.reading,
                expression_reverse: entry.expression_reverse,
                reading_reverse: entry.reading_reverse,
                sequence: entry.sequence.map(|s| s as i64),
                dictionary: entry.dictionary,
                data: data_blob,
            }
        })
        .collect();

    {
        db.begin_import_session()
            .expect("Failed to start import session");
        let conn_lock = db.conn.lock();
        let conn = conn_lock
            .unchecked_transaction()
            .expect("Failed to start transaction");

        let total_terms = serialized_terms.len();
        let mut inserted_count = 0;
        for chunk in serialized_terms.chunks(CHUNKS) {
            let mut sql = String::from("INSERT OR REPLACE INTO terms (id, expression, reading, expression_reverse, reading_reverse, sequence, dictionary, data) VALUES ");
            let placeholders: Vec<String> = (0..chunk.len())
                .map(|_| "(?, ?, ?, ?, ?, ?, ?, ?)".to_string())
                .collect();
            sql.push_str(&placeholders.join(", "));

            let mut params: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(chunk.len() * 8);
            for term in chunk {
                params.push(&term.id);
                params.push(&term.expression);
                params.push(&term.reading);
                params.push(&term.expression_reverse);
                params.push(&term.reading_reverse);
                params.push(&term.sequence);
                params.push(&term.dictionary);
                params.push(&term.data);
            }

            conn.execute(&sql, rusqlite::params_from_iter(params))
                .expect("Batch insert failed");
            inserted_count += chunk.len();
            if inserted_count % 100000 == 0 || inserted_count == total_terms {
                tracing::info!("Inserted {}/{} terms", inserted_count, total_terms);
            }
        }

        conn.commit().expect("Failed to commit");
        drop(conn_lock);
        db.end_import_session()
            .expect("Failed to end import session");
    }"""

new_block = """    let total_terms = external_data.term_list.len();
    tracing::info!("Inserting {} terms...", total_terms);

    {
        db.begin_import_session()
            .expect("Failed to start import session");
        let conn_lock = db.conn.lock();
        let conn = conn_lock
            .unchecked_transaction()
            .expect("Failed to start transaction");

        let mut inserted_count = 0;
        for chunk in external_data.term_list.chunks(CHUNKS) {
            let serialized_chunk: Vec<SerializedTerm> = chunk
                .into_par_iter()
                .map(|t| {
                    let entry = DatabaseTermEntry {
                        id: t.0.clone(),
                        expression: t.1.clone(),
                        reading: t.2.clone(),
                        expression_reverse: t.3.clone(),
                        reading_reverse: t.4.clone(),
                        definition_tags: t.5.as_ref().map(|s| s.to_string()),
                        tags: t.6.as_ref().map(|s| s.to_string()),
                        rules: t.7.to_string(),
                        score: t.8,
                        glossary: t
                            .9
                            .iter()
                            .map(|g| match g {
                                yomichan_importer::structured_content::TermGlossaryGroupType::Content(c) => {
                                    TermGlossaryGroupType::Content(TermGlossaryContentGroup {
                                        plain_text: c.plain_text.clone(),
                                        html: c.html.clone(),
                                    })
                                }
                                yomichan_importer::structured_content::TermGlossaryGroupType::Deinflection(d) => {
                                    TermGlossaryGroupType::Deinflection(TermGlossaryDeinflection {
                                        form_of: d.form_of.clone(),
                                        rules: d.rules.iter().map(|s| s.to_owned()).collect(),
                                    })
                                }
                            })
                            .collect(),
                        sequence: t.10,
                        term_tags: t.11.as_ref().map(|s| s.to_string()),
                        dictionary: t.12.clone(),
                        file_path: t.13.clone(),
                    };
                    
                    let data_blob = encode(&entry).expect("Failed to encode");
                    SerializedTerm {
                        id: entry.id,
                        expression: entry.expression,
                        reading: entry.reading,
                        expression_reverse: entry.expression_reverse,
                        reading_reverse: entry.reading_reverse,
                        sequence: entry.sequence.map(|s| s as i64),
                        dictionary: entry.dictionary,
                        data: data_blob,
                    }
                })
                .collect();

            let mut sql = String::from("INSERT OR REPLACE INTO terms (id, expression, reading, expression_reverse, reading_reverse, sequence, dictionary, data) VALUES ");
            let placeholders: Vec<String> = (0..serialized_chunk.len())
                .map(|_| "(?, ?, ?, ?, ?, ?, ?, ?)".to_string())
                .collect();
            sql.push_str(&placeholders.join(", "));

            let mut params: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(serialized_chunk.len() * 8);
            for term in &serialized_chunk {
                params.push(&term.id);
                params.push(&term.expression);
                params.push(&term.reading);
                params.push(&term.expression_reverse);
                params.push(&term.reading_reverse);
                params.push(&term.sequence);
                params.push(&term.dictionary);
                params.push(&term.data);
            }

            conn.execute(&sql, rusqlite::params_from_iter(params))
                .expect("Batch insert failed");
            inserted_count += chunk.len();
            if inserted_count % 100000 == 0 || inserted_count == total_terms {
                tracing::info!("Inserted {}/{} terms", inserted_count, total_terms);
            }
        }

        conn.commit().expect("Failed to commit");
        drop(conn_lock);
    }"""

if old_block not in content:
    print("Error: Could not find the target block to replace.")
    sys.exit(1)
    
content = content.replace(old_block, new_block)

# 3. Move end_import_session() to the very end of the function.
end_session_marker = """    tracing::info!(
        "Import finished for dictionary: {}",
        dictionary_options.name
    );
    Ok(dictionary_options)"""

end_session_fixed = """    tracing::info!(
        "Import finished for dictionary: {}",
        dictionary_options.name
    );
    db.end_import_session().expect("Failed to end import session");
    Ok(dictionary_options)"""

content = content.replace(end_session_marker, end_session_fixed)

with open('../../yomichan_rs_workspace/yomichan_rs/src/database/dictionary_importer.rs', 'w') as f:
    f.write(content)
print("Successfully patched dictionary_importer.rs")
