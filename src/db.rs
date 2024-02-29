use std::sync::{atomic::Ordering, mpsc, Arc};
use std::thread;
use std::time::Instant;

use rusqlite::{params, Connection, ToSql};
use smallvec::SmallVec;

use crate::logalang::FilterRule;
use crate::parse::{ColumnDefinition, ColumnType, ParsedRowValue, Row};
use crate::LoadingProgress;

pub enum DbResponse {
    FilterApplied {
        id: u32,
        total_filtered_rows: usize,
    },
    RowsFetched {
        id: u32,
        offset: usize,
        limit: usize,
        rows: Vec<DbLogRow>,
    },
}

pub struct DbRequest {
    pub id: u32,
    pub offset: usize,
    pub limit: usize,
    pub filters: Vec<FilterRule>,
}

#[derive(Clone, Debug)]
pub enum DbRowValue {
    String(String),
    Date(i64),
    Integer(i64),
}

pub struct DbApi {
    sender: mpsc::Sender<DbRequest>,
    receiver: mpsc::Receiver<DbResponse>,
}

impl DbApi {
    pub fn new(columns: Vec<ColumnDefinition>) -> Self {
        create_database(&columns);

        let (req_send, req_recv) = mpsc::channel();
        let (resp_send, resp_recv) = mpsc::channel();

        db_thread(columns.clone(), req_recv, resp_send);

        DbApi {
            sender: req_send,
            receiver: resp_recv,
        }
    }

    pub fn get_rows(&mut self, offset: usize, limit: usize, filters: Vec<FilterRule>) {
        self.sender
            .send(DbRequest {
                id: 0,
                offset,
                limit,
                filters,
            })
            .unwrap();
    }

    pub(crate) fn get_response(&self) -> Option<DbResponse> {
        self.receiver.try_recv().ok()
    }
}

fn db_thread(
    columns: Vec<ColumnDefinition>,
    requests: mpsc::Receiver<DbRequest>,
    responses: mpsc::Sender<DbResponse>,
) {
    thread::spawn(move || {
        let mut conn = Connection::open("threaded_batched.db").unwrap();

        while let Ok(req) = requests.recv() {
            let rows = get_rows(&mut conn, req.limit, req.offset, req.filters, &columns);

            responses
                .send(DbResponse::RowsFetched {
                    id: req.id,
                    limit: req.limit,
                    offset: req.offset,
                    rows,
                })
                .unwrap();
        }
    });
}

pub fn get_row_count() -> usize {
    let conn = Connection::open("threaded_batched.db").unwrap();
    conn.query_row("SELECT count(*) FROM row", [], |row| row.get(0))
        .unwrap()
}

pub type DbLogRow = Vec<DbRowValue>;

pub fn get_rows(
    conn: &mut Connection,
    limit: usize,
    offset: usize,
    filters: Vec<FilterRule>,
    columns: &[ColumnDefinition],
) -> Vec<DbLogRow> {
    let mut sql = String::new();
    sql += "SELECT * FROM row ";

    for filter in filters {
        sql += &filter.get_sql();
    }

    sql += " LIMIT ?1 OFFSET ?2";

    log::trace!("SQL query: {sql}");

    let mut stmt = conn.prepare(&sql).unwrap();

    let data = stmt
        .query_map(params![limit, offset], |row| {
            let mut values = Vec::new();

            values.push(DbRowValue::Integer(row.get::<_, i64>(0).unwrap()));

            for (idx, column) in columns.iter().enumerate() {
                let idx = idx + 1;

                let val = match column.column_type {
                    ColumnType::String => DbRowValue::String(row.get::<_, String>(idx).unwrap()),
                    ColumnType::Date => DbRowValue::Date(row.get::<_, i64>(idx).unwrap()),
                    ColumnType::Enumeration(_) => {
                        DbRowValue::Integer(row.get::<_, i64>(idx).unwrap())
                    }
                };

                values.push(val);
            }

            Ok(values)
        })
        .unwrap()
        .collect::<Result<Vec<DbLogRow>, _>>()
        .unwrap();

    data
}

pub fn sanitize_filter(filter: &str) -> String {
    filter.replace("'", "''")
}

fn create_database(columns: &[ColumnDefinition]) {
    let conn = Connection::open("threaded_batched.db").unwrap();
    conn.execute_batch(
        "PRAGMA journal_mode = OFF;
              PRAGMA synchronous = 0;
              PRAGMA cache_size = 1000000;
              PRAGMA locking_mode = EXCLUSIVE;",
    )
    .expect("PRAGMA");

    let mut sql = "CREATE TABLE IF NOT EXISTS row (
                id INTEGER not null primary key"
        .to_string();

    for (idx, column) in columns.iter().enumerate() {
        let col_type_string = match column.column_type {
            ColumnType::String => "TEXT",
            ColumnType::Date => "INTEGER",
            ColumnType::Enumeration(_) => "INTEGER",
        };

        sql += &format!(", Column{idx} {col_type_string} not null");
    }

    sql += ")".into();

    conn.execute(&sql, []).unwrap();
}

pub fn consumer(
    columns: usize,
    recv: mpsc::Receiver<SmallVec<[Row; 16]>>,
    batch_size: usize,
    progress: Arc<LoadingProgress>,
) {
    let mut conn = Connection::open("threaded_batched.db").unwrap();
    conn.execute_batch(
        "PRAGMA journal_mode = OFF;
              PRAGMA synchronous = 0;
              PRAGMA cache_size = 1000000;
              PRAGMA locking_mode = EXCLUSIVE;",
    )
    .expect("PRAGMA");

    let now = Instant::now();
    let mut bump = bumpalo::Bump::new();

    let conn = conn.transaction().unwrap();

    {
        let mut sql_values = format!("(NULL{}),", ",?".repeat(columns)).repeat(batch_size);
        sql_values.pop();
        let query = format!("INSERT INTO row VALUES {}", sql_values);
        let mut stmt = conn.prepare_cached(&query).unwrap();

        for rows in recv {
            let mut sql_values: Vec<&dyn ToSql> = Vec::with_capacity(batch_size * 8);
            for row in rows.iter() {
                for value in &row.values {
                    match value {
                        ParsedRowValue::String { start, end } => {
                            if *end == -1 {
                                sql_values.push(bump.alloc(&row.line[*start as usize..]))
                            } else {
                                sql_values
                                    .push(bump.alloc(&row.line[*start as usize..*end as usize]))
                            }
                        }
                        ParsedRowValue::Date(val) => sql_values.push(bump.alloc(val)),
                        ParsedRowValue::Integer(val) => sql_values.push(bump.alloc(val)),
                    }
                }
            }

            if rows.len() != batch_size {
                let mut sql = format!("(NULL{}),", ",?".repeat(columns)).repeat(rows.len());
                sql.pop();
                let query = format!("INSERT INTO row VALUES {}", sql);

                conn.execute(&query, rusqlite::params_from_iter(sql_values))
                    .unwrap();
            } else {
                stmt.execute(rusqlite::params_from_iter(sql_values))
                    .unwrap();
            }

            progress
                .rows_inserted
                .fetch_add(rows.len() as u64, Ordering::SeqCst);

            bump.reset();
        }
    }
    conn.commit().unwrap();
    log::info!("Inserting took {:.2?}", now.elapsed());
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sanitize_input() {
        let sql = "';DROP TABLE *;'";

        let sanitized = sanitize_filter(sql);

        assert_eq!(sanitized, "'';DROP TABLE *;''");
    }
}
