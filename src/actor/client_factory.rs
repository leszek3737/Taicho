use std::time::Duration;

use honcho_ai::Honcho;

use taicho::error::AppResult;
use taicho::persistence::ConnectionProfile;

pub fn build_client(profile: &ConnectionProfile, api_key: Option<&str>) -> AppResult<Honcho> {
    let params = Honcho::builder()
        .base_url(profile.base_url.clone())
        .workspace_id(profile.workspace_id.clone())
        .timeout(Duration::from_secs(profile.timeout_secs))
        .max_retries(profile.max_retries)
        .maybe_api_key(api_key.filter(|k| !k.is_empty()).map(str::to_owned))
        .build();

    Ok(Honcho::from_params(params)?)
}
