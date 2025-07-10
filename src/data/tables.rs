pub const VALUE_TABLE: &str = "CREATE TABLE IF NOT EXISTS modbus_values (
                                name TEXT PRIMARY KEY,
                                address INTEGER NOT NULL,
                                modbus_table TEXT NOT NULL,
                                slave_id INTEGER NOT NULL,
                                config TEXT
                            );";

pub const POLL_TABLE: &str = "CREATE TABLE IF NOT EXISTS modbus_polls (
                                id INTEGER PRIMARY KEY AUTOINCREMENT,
                                value_id TEXT NOT NULL REFERENCES modbus_values(id),
                                timestamp INTEGER NOT NULL,
                                value blob
                            );";

pub const AGGREGATES_TABLE: &str = "CREATE TABLE IF NOT EXISTS modbus_aggregates (
                                    id INTEGER PRIMARY KEY,
                                    value_id TEXT NOT NULL REFERENCES modbus_values(id),
                                    period TEXT NOT NULL,
                                    start INTEGER NOT NULL,
                                    finish INTEGER NOT NULL,
                                    average blob,
                                    median blob,
                                    min blob,
                                    max blob,
                                    ammount INTEGER 
                                );";
