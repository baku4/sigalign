use crate::{Result, error_msg};
use super::{SequenceProvider, Labeling, FastaReader, reverse_complement_of_nucleotide_sequence};

use rusqlite::{Connection, Statement, params};

use std::path::Path;

#[derive(Debug)]
pub struct SqliteProvider {
    total_record_count: usize,
    connection: Connection,
    sequence_buffer: Vec<u8>,
    label_buffer: String,
}

impl SequenceProvider for SqliteProvider {
    fn total_record_count(&self) -> usize {
        self.total_record_count
    }
    fn sequence_of_record(&mut self, record_index: usize) -> &[u8] {
        let sequence: Vec<u8> = self.connection.query_row(
            "SELECT sequence FROM record WHERE id = (?1)",
            params![record_index + 1],
            |row| row.get(0)
        ).unwrap();

        self.sequence_buffer = sequence;

        &self.sequence_buffer
    }
}

impl Labeling for SqliteProvider {
    fn label_of_record(&mut self, record_index: usize) -> &str {
        let label: String = self.connection.query_row(
            "SELECT label FROM record WHERE id = (?1)",
            params![record_index + 1],
            |row| row.get(0)
        ).unwrap();

        self.label_buffer = label;

        &self.label_buffer
    }
}

impl SqliteProvider {
    pub fn new_db_from_fasta(db_path: &str, fasta_file_path: &str) -> Result<Self> {
        let mut sqlite_provider = Self::new_from_db_path(db_path)?;
        sqlite_provider.create_new_table()?;

        sqlite_provider.update_rows_with_fasta(fasta_file_path)?;

        Ok(sqlite_provider)
    }
    pub fn new_db_from_fasta_of_nucleotide_with_reverse_complement(db_path: &str, fasta_file_path: &str) -> Result<Self> {
        let mut sqlite_provider = Self::new_from_db_path(db_path)?;
        sqlite_provider.create_new_table()?;

        sqlite_provider.update_rows_with_fasta_of_nucleotide_with_reverse_complement(fasta_file_path)?;

        Ok(sqlite_provider)
    }
    pub fn load_from_db_path(db_path: &str) -> Result<Self> {
        if !Self::db_file_already_exist(db_path) {
            error_msg!("Sqlite database file not exists.")
        }

        let connection = match Connection::open(db_path) {
            Ok(connection) => connection,
            Err(err) => error_msg!("{}", err), 
        };
        let total_record_count = Self::get_record_count_from_connection(&connection)?;

        Ok(
            Self {
                total_record_count,
                connection,
                sequence_buffer: Vec::new(),
                label_buffer: String::new(),
            }
        )
    }
    fn new_from_db_path(db_path: &str) -> Result<Self> {
        #[cfg(test)]
        std::fs::remove_file(db_path); // TODO: Delete this

        if Self::db_file_already_exist(db_path) {
            error_msg!("Sqlite database file already exists.")
        }

        match Connection::open(db_path) {
            Ok(connection) => {
                Ok(
                    Self {
                        total_record_count: 0,
                        connection,
                        sequence_buffer: Vec::new(),
                        label_buffer: String::new(),
                    }
                )
            },
            Err(err) => error_msg!("{}", err), 
        }
    }
    fn db_file_already_exist(db_path: &str) -> bool {
        Path::new(db_path).exists()
    }
    fn create_new_table(&self) -> Result<()> {
        match self.connection.execute(
            "CREATE TABLE record (
                id INTEGER PRIMARY KEY,
                label TEXT NOT NULL,
                sequence BLOB NOT NULL)",
            [],
        ) {
            Ok(_) => Ok(()),
            Err(error) => error_msg!("{}", error),
        }
    }

    fn update_rows_with_fasta(&mut self, fasta_file_path: &str) -> Result<()> {
        let mut stmt = self.connection.prepare(
            "INSERT INTO record (label, sequence) values (?1, ?2)"
        )?;

        let mut fasta_reader = FastaReader::from_file_path(fasta_file_path)?;

        while let Some((mut label, sequence)) = fasta_reader.next() {
            Self::execute_statement(&mut stmt, label, sequence)?;
            self.total_record_count += 1;
        }

        Ok(())
    }
    fn update_rows_with_fasta_of_nucleotide_with_reverse_complement(&mut self, fasta_file_path: &str) -> Result<()> {
        let mut stmt = self.connection.prepare(
            "INSERT INTO record (label, sequence) values (?1, ?2)"
        )?;

        let mut fasta_reader = FastaReader::from_file_path(fasta_file_path)?;

        while let Some((mut label, sequence)) = fasta_reader.next() {
            let reverse_complement_sequence = reverse_complement_of_nucleotide_sequence(&sequence);
            let mut reverse_label = label.clone();
            reverse_label.push_str(":R");
            label.push_str(":F");            

            Self::execute_statement(&mut stmt, label, sequence)?;
            Self::execute_statement(&mut stmt, reverse_label, reverse_complement_sequence)?;
            self.total_record_count += 2;
        }

        Ok(())
    }
    fn execute_statement(statement: &mut Statement, label: String, sequence: Vec<u8>) -> Result<()> {
        match statement.execute(params![label, sequence]) {
            Ok(_) => Ok(()),
            Err(error) => error_msg!("{}", error)
        }
    }
    fn get_record_count_from_connection(connection: &Connection) -> Result<usize> {
        match connection.query_row(
            "SELECT COUNT(*) FROM record",
            [],
            |row| row.get(0)
        ) {
            Ok(value) => Ok(value),
            Err(error) => error_msg!("{}", error),
        }
    }
    fn getting_sequence_statement(connection: &Connection) -> Result<Statement> {
        match connection.prepare(
            "SELECT sequence FROM record WHERE id = (?1)",
        ) {
            Ok(statement) => Ok(statement),
            Err(error) => error_msg!("{}", error),
        }
    }
    fn clear_sequence_buffer(&mut self) {
        self.sequence_buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // #[test]
    // fn print_sqlite_provider() {
    //     let fasta_file_path = "src/deprecated/tests/fasta/ERR209055.fa";

    //     let db_path = "test_sqlite.db";

    //     let mut sqlite_provider = SqliteProvider::new_db_from_fasta(db_path, fasta_file_path).unwrap();

    //     println!("{:?}", sqlite_provider);

    //     let record = sqlite_provider.sequence_of_record(1);

    //     let record_string = String::from_utf8(record.to_vec()).unwrap();

    //     let label = sqlite_provider.label_of_record(1);

    //     println!("{:?}", label);
    //     println!("{:?}", record_string);
    // }
}