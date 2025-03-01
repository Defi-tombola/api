use async_graphql::{CustomValidator, InputValueError};
use chrono::{DateTime, Duration, Utc};

pub struct TimestampValidator {
    min: i64,
    max: i64,
}

impl TimestampValidator {
    pub fn new(min: i64, max: i64) -> Self {
        TimestampValidator { min, max }
    }
}

/// Validate input timestamp used for signature.
/// Timestamp should be in RFC3339 format and be in range of specified values.
impl CustomValidator<String> for TimestampValidator {
    fn check(&self, value: &String) -> Result<(), InputValueError<String>> {
        let timestamp = DateTime::parse_from_rfc3339(value)
            .map_err(|_| InputValueError::custom("Invalid timestamp"))?;

        let min = Utc::now() - Duration::seconds(self.min);
        let max = Utc::now() + Duration::seconds(self.max);

        if timestamp < min || timestamp > max {
            return Err(InputValueError::custom("Timestamp is out of range"));
        }

        Ok(())
    }
}
