use serde::Serialize;

use crate::bot::core::healthcheck::webhook_info::WebhookCheckResult;

pub trait HealthcheckTask {
    /// Actually execute the check
    async fn execute(&mut self);
    /// Check if task execution succeeded
    fn check_result(&self) -> HealthcheckTaskResult;
    /// get result for documentation
    fn get_result(&self) -> TaskResult;
}

#[derive(Debug, Serialize, Clone)]
pub struct HealthcheckCollectedResult {
    pub healthy: bool,
    pub tasks: Vec<TaskResult>,
}
impl HealthcheckCollectedResult {
    pub fn to_json(&self) -> String {
        serde_json::to_string::<HealthcheckCollectedResult>(self).expect("Failed to encode result as json.")
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct TaskResult {
    pub name: String,
    pub result: HealthcheckResult,
}

#[derive(Debug, Serialize, Clone)]
pub enum HealthcheckResult {
    WebhookInfo(WebhookCheckResult),
    NoResult(String),
}

#[derive(Debug, Serialize, Clone)]
pub enum HealthcheckTaskResult {
    /// Task failed
    Failed,
    /// Task succeeded
    Success,
    /// Task result is ignored, document result only
    Unchecked,
}