use sqlx::PgPool;
use tracing::{debug, error};
use uuid::Uuid;

use crate::{
  db::{log::LogTypes, PaymentRepository},
  utils::encryption::encrypt_string,
};

#[derive(Clone)]
pub struct SqlxPostgresqlRepository {
  pool: PgPool,
}

impl PaymentRepository for SqlxPostgresqlRepository {
  async fn new() -> Self {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    debug!("[DB] Connecting to {}", url);
    let pool = PgPool::connect(&url).await.unwrap();
    debug!("[DB] Connected to {}", url);

    Self { pool }
  }

  async fn add_log(
    &self,
    account_id: &Uuid,
    log_type: LogTypes,
    log_data: Option<&str>,
  ) -> Result<(), sqlx::Error> {
    debug!(
      "[DB] Adding log {} {:?} {:?}",
      account_id, log_type, log_data
    );
    let (log_data, encryption_method) = match log_data {
      Some(log_data) => {
        let (log_data, encryption_method) = encrypt_string(log_data);
        (Some(log_data), Some(encryption_method as i16))
      }
      None => (None, None),
    };
    let log_type: &str = log_type.into();
    let res = sqlx::query!(
      r#"INSERT INTO logs (account_id, action, data, encryption_method) VALUES ($1, $2, $3, $4);"#,
      account_id,
      log_type,
      log_data,
      encryption_method
    )
    .execute(&self.pool)
    .await;

    if let Err(e) = res {
      error!(
        "[DB] Failed to add log {} {:?} {:?}",
        account_id, log_type, log_data
      );
      return Err(e);
    }

    debug!("[DB] Added log to account {}", account_id);

    Ok(())
  }

  async fn complete_payment(&self, payment_id: &Uuid) -> Result<(), sqlx::Error> {
    debug!("[DB] Completing payment {}", payment_id);

    let res = sqlx::query!(
      r#"UPDATE payments SET completed = TRUE WHERE id = $1;"#,
      payment_id
    );
    let res = res.execute(&self.pool).await;

    if let Err(e) = res {
      error!("[DB] Failed to complete payment {}", payment_id);
      return Err(e);
    }

    debug!("[DB] Completed payment {}", payment_id);

    Ok(())
  }

  async fn get_to_be_completed_payments(&self) -> Result<Vec<Uuid>, sqlx::Error> {
    debug!("[DB] Getting to be completed payments");

    let res = sqlx::query!( r#"SELECT id FROM payments WHERE initiated = TRUE AND completed = FALSE AND received >= amount ORDER BY updated_at ASC;"#)
        .fetch_all(&self.pool)
        .await;

    if let Err(e) = res {
      error!("[DB] Failed to get to be completed payments");
      return Err(e);
    }

    let res = res.unwrap();

    let mut payments = Vec::new();

    for row in res {
      payments.push(row.id);
    }

    debug!("[DB] Got to be completed payments {:?}", payments);

    Ok(payments)
  }

  async fn get_payment_inscriptions_content(
    &self,
    payment_id: &Uuid,
  ) -> Result<Option<Vec<(Uuid, bool, String, String)>>, sqlx::Error> {
    debug!(
      "[DB] Getting payment inscription contents for payment {}",
      payment_id
    );

    let res = sqlx::query!(
      r#"SELECT id, inscribed, target, content FROM payment_inscription_contents WHERE payment_id = $1;"#,
      payment_id
    )
    .fetch_all(&self.pool)
    .await;

    if let Err(e) = res {
      error!(
        "[DB] Failed to get payment inscription contents for payment {}",
        payment_id
      );
      return Err(e);
    }

    let res = res.unwrap();

    let mut contents = Vec::new();

    for row in res {
      contents.push((row.id, row.inscribed, row.target, row.content));
    }

    debug!(
      "[DB] Got payment inscription contents for payment {}",
      payment_id
    );

    Ok(Some(contents))
  }

  async fn add_payment_inscription_details(
    &self,
    content_id: &Uuid,
    commit: &str,
    reveal: &str,
    total_fees: f64,
  ) -> Result<(), sqlx::Error> {
    debug!("[DB] Adding inscription details for content {}", content_id);

    let res = sqlx::query!(
      r#"INSERT INTO payment_inscriptions (content, commit_tx, reveal_tx, total_fees) VALUES ($1, $2, $3, $4);"#,
      content_id,
      commit,
      reveal,
      total_fees
    )
    .execute(&self.pool)
    .await;

    if let Err(e) = res {
      error!(
        "[DB] Failed to add inscription details for content {}",
        content_id
      );
      return Err(e);
    }

    debug!("[DB] Added inscription details for content {}", content_id);

    Ok(())
  }

  async fn mark_payment_inscription_content_as_inscribed(
    &self,
    content_id: &Uuid,
  ) -> Result<(), sqlx::Error> {
    debug!(
      "[DB] Marking payment inscription content {} as inscribed",
      content_id
    );

    let res = sqlx::query!(
      r#"UPDATE payment_inscription_contents SET inscribed = TRUE WHERE id = $1;"#,
      content_id
    )
    .execute(&self.pool)
    .await;

    if let Err(e) = res {
      error!(
        "[DB] Failed to mark payment inscription content {} as inscribed",
        content_id
      );
      return Err(e);
    }

    debug!(
      "[DB] Marked payment inscription content {} as inscribed",
      content_id
    );

    Ok(())
  }
}
