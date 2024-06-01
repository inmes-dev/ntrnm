use std::fmt::format;
use anyhow::Error;
use chrono::{NaiveDateTime};
use sqlx::{FromRow, PgPool};

#[derive(Debug, FromRow)]
pub struct SimpleMessageRecord {
    pub id: i64,
    pub name: String,
    pub seq: i64,
    pub last_seq: i64,
    pub latest_msg_time: NaiveDateTime,
}

const TABLE_NAME: &'static str = "latest_simple_group_messages";

impl SimpleMessageRecord {
    pub async fn create_table(pool: &PgPool) -> Result<(), Error> {
        let exists: (bool,) = sqlx::query_as(format!("SELECT EXISTS ( \
            SELECT 1 \
            FROM information_schema.tables \
            WHERE table_schema = 'public' AND TABLE_NAME = '{}' \
        )", TABLE_NAME).as_str()).fetch_one(pool).await?;
        if !exists.0 {
            sqlx::query(format!("CREATE TABLE {} ( \
                id BIGINT PRIMARY KEY, \
                name VARCHAR(255) NOT NULL, \
                seq BIGINT NOT NULL, \
                last_seq BIGINT NOT NULL,\
                latest_msg_time TIMESTAMP NOT NULL \
            )", TABLE_NAME).as_str()).execute(pool).await?;
        }
        Ok(())
    }

    pub async fn insert(pool: &PgPool, message: SimpleMessageRecord) -> Result<(), Error> {
        sqlx::query(format!(r#"
            INSERT INTO "public"."{}" ("id", "name", "seq", "last_seq", "latest_msg_time")
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT ("id") DO UPDATE SET
                "name" = EXCLUDED."name",
                "seq" = EXCLUDED."seq",
                "last_seq" = EXCLUDED."last_seq",
                "latest_msg_time" = EXCLUDED."latest_msg_time"
        "#, TABLE_NAME).as_str())
            .bind(message.id)
            .bind(&message.name)
            .bind(message.seq)
            .bind(message.last_seq)
            .bind(message.latest_msg_time)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn get_by_id(pool: &PgPool, id: i64) -> Result<SimpleMessageRecord, Error> {
        let message = sqlx::query_as::<_, SimpleMessageRecord>(format!(
            "SELECT id, name, seq, last_seq, latest_msg_time FROM {} WHERE id = $1", TABLE_NAME
        ).as_str())
            .bind(id)
            .fetch_one(pool)
            .await?;
        Ok(message)
    }
}