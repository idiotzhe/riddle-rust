import { Database } from "bun:sqlite";
import path from 'path';

const dbPath = path.join(import.meta.dir, 'lantern.db');
const db = new Database(dbPath);

// Initialize tables if they don't exist
db.run(`
  CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    avatar TEXT,
    user_code TEXT UNIQUE,
    token TEXT UNIQUE,
    register_time DATETIME DEFAULT CURRENT_TIMESTAMP
  );
`);

db.run(`
  CREATE TABLE IF NOT EXISTS riddles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    question TEXT NOT NULL,
    remark TEXT,
    options_json TEXT NOT NULL DEFAULT '[]',
    answer TEXT NOT NULL,
    add_time DATETIME DEFAULT CURRENT_TIMESTAMP,
    is_solved BOOLEAN DEFAULT 0,
    solver_id INTEGER,
    FOREIGN KEY (solver_id) REFERENCES users(id)
  );
`);

db.run(`
  CREATE TABLE IF NOT EXISTS activities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    start_time DATETIME NOT NULL,
    end_time DATETIME NOT NULL
  );
`);

db.run(`
  CREATE TABLE IF NOT EXISTS guess_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    riddle_id INTEGER NOT NULL,
    solve_time DATETIME DEFAULT CURRENT_TIMESTAMP,
    is_solved BOOLEAN DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (riddle_id) REFERENCES riddles(id),
    UNIQUE(user_id, riddle_id)
  );
`);

export default db;
