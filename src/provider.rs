use async_trait::async_trait;
use configcat::{Client, ClientError, ErrorKind, User, UserValue};
use open_feature::provider::{FeatureProvider, ProviderMetadata, ResolutionDetails};
use open_feature::{
    EvaluationContext, EvaluationContextFieldValue, EvaluationError, EvaluationErrorCode,
    EvaluationReason, EvaluationResult, StructValue, Value,
};

const NAME: &str = "ConfigCatProvider";

/// The ConfigCat OpenFeature provider.
///
/// # Examples
///
/// ```no_run
/// use std::time::Duration;
/// use configcat::{Client, PollingMode};
/// use open_feature::OpenFeature;
/// use configcat_openfeature_provider::ConfigCatProvider;
///
/// #[tokio::main]
/// async fn main() {
///     // Acquire an OpenFeature API instance.
///     let mut api = OpenFeature::singleton_mut().await;
///
///     // Configure the ConfigCat SDK.
///     let configcat_client = Client::builder("sdk-key")
///         .polling_mode(PollingMode::AutoPoll(Duration::from_secs(60)))
///         .build()
///         .unwrap();
///
///     // Configure the provider.
///     api.set_provider(ConfigCatProvider::new(configcat_client)).await;
///
///     // Create a client.
///     let client = api.create_client();
///
///     // Evaluate a feature flag.
///     let is_awesome_feature_enabled = client
///         .get_bool_value("isAwesomeFeatureEnabled", None, None)
///         .await
///         .unwrap_or(false);
/// }
/// ```
pub struct ConfigCatProvider {
    client: Client,
    provider_metadata: ProviderMetadata,
}

impl ConfigCatProvider {
    /// The ConfigCat OpenFeature provider.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    /// use configcat::{Client, PollingMode};
    /// use configcat_openfeature_provider::ConfigCatProvider;
    ///
    /// let configcat_client = Client::builder("sdk-key")
    ///     .polling_mode(PollingMode::AutoPoll(Duration::from_secs(60)))
    ///     .build()
    ///     .unwrap();
    ///
    /// let provider = ConfigCatProvider::new(configcat_client);
    /// ```
    pub fn new(client: Client) -> Self {
        Self {
            client,
            provider_metadata: ProviderMetadata::new(NAME),
        }
    }
}

#[async_trait]
impl FeatureProvider for ConfigCatProvider {
    fn metadata(&self) -> &ProviderMetadata {
        &self.provider_metadata
    }

    async fn resolve_bool_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<bool>> {
        match to_user(evaluation_context) {
            Ok(user) => {
                let details = self.client.get_value_details(flag_key, false, user).await;
                to_res_details(&details)
            }
            Err(err) => Err(err),
        }
    }

    async fn resolve_int_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<i64>> {
        match to_user(evaluation_context) {
            Ok(user) => {
                let details = self.client.get_value_details(flag_key, 0, user).await;
                to_res_details(&details)
            }
            Err(err) => Err(err),
        }
    }

    async fn resolve_float_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<f64>> {
        match to_user(evaluation_context) {
            Ok(user) => {
                let details = self.client.get_value_details(flag_key, 0.0, user).await;
                to_res_details(&details)
            }
            Err(err) => Err(err),
        }
    }

    async fn resolve_string_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<String>> {
        match to_user(evaluation_context) {
            Ok(user) => {
                let details = self
                    .client
                    .get_value_details(flag_key, String::default(), user)
                    .await;
                to_res_details(&details)
            }
            Err(err) => Err(err),
        }
    }

    async fn resolve_struct_value(
        &self,
        flag_key: &str,
        evaluation_context: &EvaluationContext,
    ) -> EvaluationResult<ResolutionDetails<StructValue>> {
        match to_user(evaluation_context) {
            Ok(user) => {
                let details = self
                    .client
                    .get_value_details(flag_key, String::default(), user)
                    .await;
                to_struct_details(&details)
            }
            Err(err) => Err(err),
        }
    }
}

fn to_user(ctx: &EvaluationContext) -> Result<Option<User>, EvaluationError> {
    if ctx.targeting_key.is_none() && ctx.custom_fields.is_empty() {
        return Ok(None);
    }
    let identifier = match ctx.targeting_key.as_ref() {
        Some(id) => id,
        None => "",
    };
    let mut user = User::new(identifier);
    for (key, attr) in &ctx.custom_fields {
        match key.as_str() {
            User::EMAIL => {
                if let Some(email) = attr.as_str() {
                    user = user.email(email);
                }
            }
            User::COUNTRY => {
                if let Some(country) = attr.as_str() {
                    user = user.country(country);
                }
            }
            _ => {
                if let Some(attr_val) = to_user_value(attr) {
                    user = user.custom(key, attr_val);
                } else {
                    return Err(EvaluationError::builder()
                        .code(EvaluationErrorCode::InvalidContext)
                        .message(format!(
                            "{key} context attribute is not supported by the ConfigCat Provider."
                        ))
                        .build());
                }
            }
        }
    }
    Ok(Some(user))
}

fn to_user_value(val: &EvaluationContextFieldValue) -> Option<UserValue> {
    match val {
        EvaluationContextFieldValue::Bool(val) => Some(UserValue::String(val.to_string())),
        EvaluationContextFieldValue::Int(val) => Some(UserValue::Int(*val)),
        EvaluationContextFieldValue::Float(val) => Some(UserValue::Float(*val)),
        EvaluationContextFieldValue::String(val) => Some(UserValue::String(val.to_owned())),
        EvaluationContextFieldValue::DateTime(val) => Some(UserValue::Int(val.unix_timestamp())),
        EvaluationContextFieldValue::Struct(_) => None,
    }
}

fn to_res_details<T: Clone>(
    details: &configcat::EvaluationDetails<T>,
) -> EvaluationResult<ResolutionDetails<T>> {
    if let Some(err) = &details.error {
        return Err(to_res_error(err));
    }
    let reason = construct_reason(details);
    Ok(ResolutionDetails {
        value: details.value.clone(),
        reason: Some(reason),
        variant: details.variation_id.clone(),
        flag_metadata: None,
    })
}

fn to_struct_details(
    details: &configcat::EvaluationDetails<String>,
) -> EvaluationResult<ResolutionDetails<StructValue>> {
    if let Some(err) = &details.error {
        return Err(to_res_error(err));
    }
    let json_val: serde_json::Value = match serde_json::from_str(details.value.as_str()) {
        Ok(val) => val,
        Err(err) => {
            return Err(EvaluationError::builder()
                .code(EvaluationErrorCode::ParseError)
                .message(format!("Failed to parse JSON from evaluated string: {err}"))
                .build())
        }
    };
    let val: Value = match json_val.try_into() {
        Ok(val) => val,
        Err(err) => return Err(err),
    };
    return match val.as_struct() {
        Some(struct_val) => {
            let reason = construct_reason(details);
            Ok(ResolutionDetails {
                value: struct_val.clone(),
                reason: Some(reason),
                variant: details.variation_id.clone(),
                flag_metadata: None,
            })
        }
        None => Err(EvaluationError::builder()
            .code(EvaluationErrorCode::TypeMismatch)
            .message("Parsed value is not a StructValue")
            .build()),
    };
}

fn to_res_error(err: &ClientError) -> EvaluationError {
    match err.kind {
        ErrorKind::ConfigJsonNotAvailable => EvaluationError::builder()
            .code(EvaluationErrorCode::ParseError)
            .message(&err.message)
            .build(),
        ErrorKind::SettingKeyMissing => EvaluationError::builder()
            .code(EvaluationErrorCode::FlagNotFound)
            .message(&err.message)
            .build(),
        ErrorKind::SettingValueTypeMismatch => EvaluationError::builder()
            .code(EvaluationErrorCode::TypeMismatch)
            .message(&err.message)
            .build(),
        _ => EvaluationError::builder()
            .code(EvaluationErrorCode::General("Provider error".to_owned()))
            .message(&err.message)
            .build(),
    }
}

fn construct_reason<T>(details: &configcat::EvaluationDetails<T>) -> EvaluationReason {
    if details.matched_percentage_option.is_some() || details.matched_targeting_rule.is_some() {
        return EvaluationReason::TargetingMatch;
    }
    EvaluationReason::Default
}
