#!/bin/bash
cat ../../yomichan_rs_workspace/yomichan_rs/src/database/dictionary_importer.rs | awk '
BEGIN { skipping = 0 }
/let serialized_terms: Vec<SerializedTerm> = external_data/ {
    skipping = 1
    print "    let total_terms = external_data.term_list.len();"
    print "    {"
    print "        db.begin_import_session().expect(\"Failed to start import session\");"
    print "        let conn_lock = db.conn.lock();"
    print "        let conn = conn_lock.unchecked_transaction().expect(\"Failed to start transaction\");"
    print ""
    print "        let mut inserted_count = 0;"
    print "        for chunk in external_data.term_list.chunks(CHUNKS) {"
    print "            let serialized_chunk: Vec<SerializedTerm> = chunk"
    print "                .into_par_iter()"
    print "                .map(|t| {"
    print "                    let entry = DatabaseTermEntry {"
    print "                        id: t.0.clone(),"
    print "                        expression: t.1.clone(),"
    print "                        reading: t.2.clone(),"
    print "                        expression_reverse: t.3.clone(),"
    print "                        reading_reverse: t.4.clone(),"
    print "                        definition_tags: t.5.as_ref().map(|s| s.to_string()),"
    print "                        tags: t.6.as_ref().map(|s| s.to_string()),"
    print "                        rules: t.7.to_string(),"
    print "                        score: t.8,"
    print "                        glossary: t.9.iter().map(|g| match g {"
    print "                            yomichan_importer::structured_content::TermGlossaryGroupType::Content(c) => {"
    print "                                TermGlossaryGroupType::Content(TermGlossaryContentGroup {"
    print "                                    plain_text: c.plain_text.clone(),"
    print "                                    html: c.html.clone(),"
    print "                                })"
    print "                            }"
    print "                            yomichan_importer::structured_content::TermGlossaryGroupType::Deinflection(d) => {"
    print "                                TermGlossaryGroupType::Deinflection(TermGlossaryDeinflection {"
    print "                                    form_of: d.form_of.clone(),"
    print "                                    rules: d.rules.iter().map(|s| s.to_owned()).collect(),"
    print "                                })"
    print "                            }"
    print "                        }).collect(),"
    print "                        sequence: t.10,"
    print "                        term_tags: t.11.as_ref().map(|s| s.to_string()),"
    print "                        dictionary: t.12.clone(),"
    print "                        file_path: t.13.clone(),"
    print "                    };"
    print "                    let data_blob = encode(&entry).expect(\"Failed to encode\");"
    print "                    SerializedTerm {"
    print "                        id: entry.id,"
    print "                        expression: entry.expression,"
    print "                        reading: entry.reading,"
    print "                        expression_reverse: entry.expression_reverse,"
    print "                        reading_reverse: entry.reading_reverse,"
    print "                        sequence: entry.sequence.map(|s| s as i64),"
    print "                        dictionary: entry.dictionary,"
    print "                        data: data_blob,"
    print "                    }"
    print "                }).collect();"
    print ""
    print "            let mut sql = String::from(\"INSERT OR REPLACE INTO terms (id, expression, reading, expression_reverse, reading_reverse, sequence, dictionary, data) VALUES \");"
    print "            let placeholders: Vec<String> = (0..serialized_chunk.len()).map(|_| \"(?, ?, ?, ?, ?, ?, ?, ?)\".to_string()).collect();"
    print "            sql.push_str(&placeholders.join(\", \"));"
    print ""
    print "            let mut params: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(serialized_chunk.len() * 8);"
    print "            for term in &serialized_chunk {"
    print "                params.push(&term.id);"
    print "                params.push(&term.expression);"
    print "                params.push(&term.reading);"
    print "                params.push(&term.expression_reverse);"
    print "                params.push(&term.reading_reverse);"
    print "                params.push(&term.sequence);"
    print "                params.push(&term.dictionary);"
    print "                params.push(&term.data);"
    print "            }"
    print ""
    print "            conn.execute(&sql, rusqlite::params_from_iter(params)).expect(\"Batch insert failed\");"
    print "            inserted_count += chunk.len();"
    print "            if inserted_count % 100000 == 0 || inserted_count == total_terms {"
    print "                tracing::info!(\"Inserted {}/{} terms\", inserted_count, total_terms);"
    print "            }"
    print "        }"
    print "        conn.commit().expect(\"Failed to commit\");"
    print "        drop(conn_lock);"
    print "    }"
    next
}
/{ db.begin_import_session()/ {
    if (skipping) {
        skipping = 2 # Inside the old insertion block
    }
}
/^    }$/ {
    if (skipping == 2) {
        skipping = 0
        next
    }
}
{
    if (skipping == 0) {
        print $0
    }
}
' > temp_importer.rs
mv temp_importer.rs ../../yomichan_rs_workspace/yomichan_rs/src/database/dictionary_importer.rs
