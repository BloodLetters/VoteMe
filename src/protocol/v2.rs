use std::collections::HashMap;

use base64::prelude::*;
use serde::{Deserialize, Serialize};

use crate::crypto::hmac::verify_hmac_signature;
use crate::error::{VotifierError, VotifierResult};
use crate::model::VoteRequest;

#[derive(Debug, Serialize, Deserialize)]
pub struct V2OuterMessage {
    pub payload: String,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct V2InnerPayload {
    pub service_name: String,
    pub username: String,
    pub address: String,
    pub timestamp: String,
    pub challenge: String,
}

pub fn parse_v2_packet(
    data: &[u8],
    tokens: &HashMap<String, String>,
    expected_challenge: &str,
) -> VotifierResult<VoteRequest> {
    let raw_text = std::str::from_utf8(data)
        .map_err(|e| VotifierError::InvalidPayload(format!("Invalid UTF-8 in V2 payload: {e}")))?
        .trim();

    let json_start = raw_text
        .find('{')
        .ok_or_else(|| VotifierError::InvalidPayload("Missing opening brace in V2 JSON".to_string()))?;
    let json_end = raw_text
        .rfind('}')
        .ok_or_else(|| VotifierError::InvalidPayload("Missing closing brace in V2 JSON".to_string()))?;

    if json_start > json_end {
        return Err(VotifierError::InvalidPayload(
            "Malformed JSON braces structure in V2".to_string(),
        ));
    }

    let json_str = &raw_text[json_start..=json_end];

    let outer: V2OuterMessage = serde_json::from_str(json_str)
        .map_err(|e| VotifierError::InvalidPayload(format!("Failed to parse outer JSON: {e}")))?;

    let provided_sig = BASE64_STANDARD
        .decode(outer.signature.trim())
        .map_err(|e| VotifierError::InvalidPayload(format!("Signature is not valid Base64: {e}")))?;

    let inner: V2InnerPayload = serde_json::from_str(&outer.payload)
        .map_err(|e| VotifierError::InvalidPayload(format!("Failed to parse inner JSON: {e}")))?;

    if inner.service_name.trim().is_empty() {
        return Err(VotifierError::InvalidPayload("Empty serviceName in V2 payload".into()));
    }
    if inner.username.trim().is_empty() {
        return Err(VotifierError::InvalidPayload("Empty username in V2 payload".into()));
    }
    if inner.timestamp.trim().is_empty() {
        return Err(VotifierError::InvalidPayload("Empty timestamp in V2 payload".into()));
    }

    let token = tokens
        .get(&inner.service_name)
        .or_else(|| tokens.get("default"))
        .ok_or_else(|| {
            VotifierError::AuthenticationFailed(format!(
                "No secret token configured for service '{}' or default",
                inner.service_name
            ))
        })?;

    let signature_valid = verify_hmac_signature(
        &provided_sig,
        outer.payload.as_bytes(),
        token.as_bytes(),
    )?;

    if !signature_valid {
        return Err(VotifierError::AuthenticationFailed(format!(
            "Signature verification failed for service '{}'",
            inner.service_name
        )));
    }

    if inner.challenge.trim() != expected_challenge.trim() {
        return Err(VotifierError::AuthenticationFailed(
            "Handshake challenge token mismatch".to_string(),
        ));
    }

    Ok(VoteRequest {
        service_name: inner.service_name,
        username: inner.username,
        address: inner.address,
        timestamp: inner.timestamp,
    })
}

pub fn format_v2_packet(
    payload: &V2InnerPayload,
    token: &str,
) -> VotifierResult<String> {
    let inner_json = serde_json::to_string(payload)?;
    let signature_bytes = crate::crypto::hmac::compute_hmac_sha256(inner_json.as_bytes(), token.as_bytes())?;
    let signature_b64 = BASE64_STANDARD.encode(signature_bytes);

    let outer = V2OuterMessage {
        payload: inner_json,
        signature: signature_b64,
    };

    Ok(serde_json::to_string(&outer)?)
}
