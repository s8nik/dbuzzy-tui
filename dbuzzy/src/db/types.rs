pub struct DatabaseRow(String);

impl AsRef<str> for DatabaseRow {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<tokio_postgres::Row> for DatabaseRow {
    fn from(row: tokio_postgres::Row) -> Self {
        Self(row.get("datname"))
    }
}

pub struct TableRow {
    pub name: String,
    pub schema: String,
}

impl From<tokio_postgres::Row> for TableRow {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            name: row.get("table_name"),
            schema: row.get("table_schema"),
        }
    }
}

pub struct ColumnRow {
    pub name: String,
    pub r#type: String,
    pub is_nullable: bool,
    pub default: String,
}

impl From<tokio_postgres::Row> for ColumnRow {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            name: row.get("column_name"),
            r#type: row.get("data_type"),
            is_nullable: row.get("is_nullable"),
            default: row.get("column_default"),
        }
    }
}
