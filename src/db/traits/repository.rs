use uuid::Uuid;

use crate::db::log::LogTypes;

pub trait PaymentRepository
where
  Self: Clone,
{
  async fn new() -> Self;

  async fn add_log(
    &self,
    account_id: &Uuid,
    log_type: LogTypes,
    log_data: Option<&str>,
  ) -> Result<(), sqlx::Error>;

  async fn complete_payment(&self, payment_id: &Uuid) -> Result<(), sqlx::Error>;

  async fn get_to_be_completed_payments(&self) -> Result<Vec<Uuid>, sqlx::Error>;

  async fn get_payment_inscriptions_content(
    &self,
    payment_id: &Uuid,
  ) -> Result<Option<Vec<(Uuid, bool, String, String)>>, sqlx::Error>;

  async fn add_payment_inscription_details(
    &self,
    content_id: &Uuid,
    commit: &str,
    reveal: &str,
    total_fees: f64,
  ) -> Result<(), sqlx::Error>;

  async fn mark_payment_inscription_content_as_inscribed(
    &self,
    content_id: &Uuid,
  ) -> Result<(), sqlx::Error>;
}
