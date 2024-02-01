use std::sync::mpsc;
use std::thread;
use std::time::Instant;

use rusqlite::{params, Connection, ToSql};

use crate::parse::LogRow;

pub struct DbResponse {
    pub id: u32,
    pub offset: usize,
    pub limit: usize,
    pub rows: Vec<DbLogRow>,
}

pub struct DbRequest {
    pub id: u32,
    pub offset: usize,
    pub limit: usize,
}

pub struct DbApi {
    sender: mpsc::Sender<DbRequest>,
    receiver: mpsc::Receiver<DbResponse>,
}

impl DbApi {
    pub fn new() -> Self {
        let (req_send, req_recv) = mpsc::channel();
        let (resp_send, resp_recv) = mpsc::channel();

        db_thread(req_recv, resp_send);

        DbApi {
            sender: req_send,
            receiver: resp_recv,
        }
    }

    pub fn get_rows(&mut self, offset: usize, limit: usize) {
        self.sender
            .send(DbRequest {
                id: 0,
                offset,
                limit,
            })
            .unwrap();
    }

    pub(crate) fn get_response(&self) -> Option<DbResponse> {
        self.receiver.try_recv().ok()
    }
}

fn db_thread(requests: mpsc::Receiver<DbRequest>, responses: mpsc::Sender<DbResponse>) {
    thread::spawn(move || {
        let mut conn = Connection::open("threaded_batched.db").unwrap();

        while let Ok(req) = requests.recv() {
            let rows = get_rows(&mut conn, req.limit, req.offset);

            responses
                .send(DbResponse {
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
    let mut conn = Connection::open("threaded_batched.db").unwrap();
    conn.query_row("SELECT count(*) FROM row", [], |row| row.get(0))
        .unwrap()
}

pub struct DbLogRow {
    pub id: i64,
    pub time: i64,
    pub level: i8,
    pub context: String,
    pub thread: String,
    pub file: String,
    pub method: String,
    pub object: String,
    pub message: String,
}

pub fn get_rows(conn: &mut Connection, limit: usize, offset: usize) -> Vec<DbLogRow> {
    let mut stmt = conn
        .prepare("SELECT id, time, level, context, thread, file, method, object, message FROM row LIMIT ?1 OFFSET ?2")
        .unwrap();

    let data = stmt
        .query_map(params![limit, offset], |row| {
            Ok(DbLogRow {
                id: row.get::<_, i64>(0).unwrap(),
                time: row.get::<_, i64>(1).unwrap(),
                level: row.get::<_, i8>(2).unwrap(),
                context: row.get::<_, String>(3).unwrap(),
                thread: row.get::<_, String>(4).unwrap(),
                file: row.get::<_, String>(5).unwrap(),
                method: row.get::<_, String>(6).unwrap(),
                object: row.get::<_, String>(7).unwrap(),
                message: row.get::<_, String>(8).unwrap(),
            })
        })
        .unwrap()
        .collect::<Result<Vec<DbLogRow>, _>>()
        .unwrap();

    data
}

pub fn consumer(recv: mpsc::Receiver<Vec<LogRow>>, batch_size: usize) {
    let mut conn = Connection::open("threaded_batched.db").unwrap();
    conn.execute_batch(
        "PRAGMA journal_mode = OFF;
              PRAGMA synchronous = 0;
              PRAGMA cache_size = 1000000;
              PRAGMA locking_mode = EXCLUSIVE;",
    )
    .expect("PRAGMA");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS row (
                id INTEGER not null primary key,
                time INTEGER not null,
                level INTEGER not null,
                context TEXT not null,
                thread TEXT not null,
                file TEXT not null,
                method TEXT not null,
                object TEXT not null,
                message TEXT not null)",
        [],
    )
    .unwrap();
    conn.execute("CREATE INDEX idx_log_time ON row (time)", [])
        .unwrap();

    let now = Instant::now();
    let mut bump = bumpalo::Bump::new();

    let conn = conn.transaction().unwrap();

    {
        let mut sql_values = "(NULL, ?, ?, ?, ?, ?, ?, ?, ?),".repeat(batch_size);
        sql_values.pop();
        let query = format!("INSERT INTO row VALUES {}", sql_values);
        let mut stmt = conn.prepare_cached(&query).unwrap();

        for rows in recv {
            let mut row_values: Vec<&dyn ToSql> = Vec::with_capacity(batch_size * 8);

            for row in rows.iter() {
                row_values.push(bump.alloc(row.time_unixtime));
                row_values.push(bump.alloc(row.level));
                row_values.push(bump.alloc(row.context()));
                row_values.push(bump.alloc(row.thread()));
                row_values.push(bump.alloc(row.file()));
                row_values.push(bump.alloc(row.method()));
                row_values.push(bump.alloc(row.object()));
                row_values.push(bump.alloc(row.message()));
            }

            stmt.execute(rusqlite::params_from_iter(row_values))
                .unwrap();

            bump.reset();
        }
    }
    conn.commit().unwrap();
    println!("Inserting took {:.2?}", now.elapsed());
}
