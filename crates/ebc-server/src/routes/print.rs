//! Print orders (`docs/api.md` §4.6 and §5): the postal address a partner sends, the
//! order forwarded to the print provider with the card the partner uploaded, and the
//! printer's refusals passed through.

use axum::http::StatusCode;
use serde::Deserialize;

use super::{MAX_JSON_BYTES, parse_json};
use crate::error::ApiError;
use crate::state::AppState;
use crate::store::{GiftRecord, PrintOrderRecord};

/// Body of `POST /api/partner/v1/gifts/{id}/print-orders`.
#[derive(Debug, Deserialize)]
struct PrintOrderRequest {
    product: String,
    pdf_sha256: String,
    address: serde_json::Value,
    #[serde(default)]
    quantity: Option<u32>,
}

/// Country used when the address does not name one.
pub(crate) const DEFAULT_COUNTRY: &str = "FR";

/// A postal address as the print provider takes it (`docs/api.md` §3 and §5).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct PostalAddress {
    /// Addressee, 1 to 80 characters.
    pub(crate) name: String,
    /// First address line, 1 to 120 characters.
    pub(crate) line1: String,
    /// Optional second line, at most 120 characters; empty is omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) line2: Option<String>,
    /// Postal code, 1 to 12 characters.
    pub(crate) postal_code: String,
    /// City, 1 to 80 characters.
    pub(crate) city: String,
    /// ISO 3166-1 alpha-2 code, upper case; `FR` when absent.
    pub(crate) country: String,
}

/// The members of an address and their maximum length in characters; `true` when the
/// member is required.
const ADDRESS_FIELDS: [(&str, usize, bool); 6] = [
    ("name", 80, true),
    ("line1", 120, true),
    ("line2", 120, false),
    ("postal_code", 12, true),
    ("city", 80, true),
    ("country", 2, false),
];

/// `400 invalid_address` naming the offending member.
fn bad_address(field: &str, message: String) -> ApiError {
    ApiError::bad_request("invalid_address", message)
        .with_extra("field", format!("address.{field}").into())
}

impl PostalAddress {
    /// Validates the `address` member of a print order. Text is trimmed; control
    /// characters, unknown members and wrong types are refused; the country defaults to
    /// [`DEFAULT_COUNTRY`] and is upper-cased. Whether the printer posts to that country
    /// is the printer's decision (`unsupported_country`).
    ///
    /// # Errors
    /// `400 invalid_address` with `field` (e.g. `"address.postal_code"`).
    pub(crate) fn from_json(value: &serde_json::Value) -> Result<Self, ApiError> {
        let object = value.as_object().ok_or_else(|| {
            ApiError::bad_request(
                "invalid_address",
                "address must be an object {name, line1, line2?, postal_code, city, country}",
            )
            .with_extra("field", "address".into())
        })?;
        if let Some(unknown) =
            object.keys().find(|key| !ADDRESS_FIELDS.iter().any(|(name, ..)| name == key))
        {
            let shown: String = unknown.chars().filter(|c| !c.is_control()).take(40).collect();
            return Err(bad_address(
                &shown,
                format!(
                    "address.{shown} is not a member; expected name, line1, line2, postal_code, city, country"
                ),
            ));
        }
        let mut texts: [Option<String>; 6] = Default::default();
        for ((field, max_chars, required), slot) in ADDRESS_FIELDS.iter().zip(texts.iter_mut()) {
            let text = match object.get(*field) {
                None | Some(serde_json::Value::Null) => None,
                Some(serde_json::Value::String(text)) => {
                    Some(text.trim()).filter(|text| !text.is_empty())
                }
                Some(_) => {
                    return Err(bad_address(field, format!("address.{field} must be a string")));
                }
            };
            match text {
                None if *required => {
                    return Err(bad_address(field, format!("address.{field} is required")));
                }
                None => {}
                Some(text) => {
                    if text.chars().any(char::is_control) {
                        return Err(bad_address(
                            field,
                            format!("address.{field} must not contain control characters"),
                        ));
                    }
                    if text.chars().count() > *max_chars {
                        return Err(bad_address(
                            field,
                            format!("address.{field} is longer than {max_chars} characters"),
                        ));
                    }
                    *slot = Some(text.to_owned());
                }
            }
        }
        let [name, line1, line2, postal_code, city, country] = texts;
        let country = country.unwrap_or_else(|| DEFAULT_COUNTRY.to_owned()).to_ascii_uppercase();
        if country.len() != 2 || !country.bytes().all(|b| b.is_ascii_uppercase()) {
            return Err(bad_address(
                "country",
                "address.country must be an ISO 3166-1 alpha-2 code such as FR".to_owned(),
            ));
        }
        let required = |value: Option<String>| value.unwrap_or_default();
        Ok(Self {
            name: required(name),
            line1: required(line1),
            line2,
            postal_code: required(postal_code),
            city: required(city),
            country,
        })
    }
}

/// Largest printer answer read, in bytes.
const MAX_PRINTER_ANSWER_BYTES: usize = MAX_JSON_BYTES * 4;

/// A short, printable copy of a text member of the printer's answer.
fn printer_text(value: Option<&serde_json::Value>, max_chars: usize) -> Option<String> {
    let text: String =
        value?.as_str()?.chars().filter(|c| !c.is_control()).take(max_chars).collect();
    (!text.trim().is_empty()).then_some(text)
}

/// The error the caller gets when the printer refuses an order with a `4xx`: the printer's
/// own `error.code` is passed through (`docs/api.md` §3).
///
/// * `preflight_failed` stays `422 preflight_failed` (the card itself cannot be printed);
/// * anything else becomes `422 printer_rejected`.
///
/// Both carry `printer_status`, `printer_code`, `printer_message` (when the printer gave
/// one) and `findings` (the printer's, or `[]`).
fn printer_refusal(status: StatusCode, answer: &serde_json::Value) -> ApiError {
    let printer_code = printer_text(answer.pointer("/error/code"), 64)
        .filter(|code| {
            code.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
        })
        .unwrap_or_else(|| format!("http_{}", status.as_u16()));
    let printer_message = printer_text(answer.pointer("/error/message"), 300);
    let findings = answer
        .pointer("/error/findings")
        .filter(|findings| findings.is_array())
        .cloned()
        .unwrap_or_else(|| serde_json::json!([]));
    let mut error = if printer_code == "preflight_failed" {
        ApiError::unprocessable("preflight_failed", "the print provider refused the PDF")
    } else {
        let detail = printer_message.as_deref().unwrap_or("no reason given");
        ApiError::unprocessable(
            "printer_rejected",
            format!("the print provider refused the order ({printer_code}): {detail}"),
        )
    };
    error = error
        .with_extra("printer_status", status.as_u16().into())
        .with_extra("printer_code", printer_code.into())
        .with_extra("findings", findings);
    if let Some(message) = printer_message {
        error = error.with_extra("printer_message", message.into());
    }
    error
}

/// `POST /api/partner/v1/gifts/{id}/print-orders` (after authentication): forwards the
/// card the partner uploaded (`PUT …/card.pdf`) to the print provider (`EBC_PRINTER_URL`,
/// `docs/api.md` §5) as `multipart/form-data`.
///
/// The printer's `4xx` answers come back as `422 preflight_failed` / `422
/// printer_rejected` with the printer's code ([`printer_refusal`]); no answer, a `5xx` or
/// an unusable answer is `502 printer_unavailable`.
pub(crate) async fn place_print_order(
    state: &AppState,
    gift: &GiftRecord,
    body: &[u8],
) -> Result<serde_json::Value, ApiError> {
    if gift.status_at(state.now()).is_gone() {
        return Err(ApiError::conflict("gift_gone", "the gift is cancelled or expired"));
    }
    let order: PrintOrderRequest = parse_json(body)?;
    let id = gift.id.clone();
    let product_ok = (1..=64).contains(&order.product.len())
        && order
            .product
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'));
    if !product_ok {
        return Err(ApiError::unprocessable("invalid_request", "product must be a product id"));
    }
    let address = PostalAddress::from_json(&order.address)?;
    let quantity = order.quantity.unwrap_or(1);
    if !(1..=1000).contains(&quantity) {
        return Err(ApiError::unprocessable(
            "invalid_request",
            "quantity must be between 1 and 1000",
        ));
    }
    let printer = state.config.printer_url.as_deref().ok_or_else(|| {
        ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "printing_not_configured",
            "no print provider is configured",
        )
    })?;
    let card = gift.card_pdf.as_ref().ok_or_else(|| {
        ApiError::conflict("card_pdf_missing", "upload the card with PUT …/card.pdf first")
    })?;
    if !order.pdf_sha256.eq_ignore_ascii_case(&card.sha256) {
        return Err(ApiError::conflict(
            "pdf_mismatch",
            "pdf_sha256 does not match the uploaded card",
        ));
    }
    let pdf = state
        .read_blob(&card.sha256)
        .await?
        .ok_or_else(|| ApiError::internal("the card PDF is missing"))?;

    let unavailable =
        |detail: String| ApiError::new(StatusCode::BAD_GATEWAY, "printer_unavailable", detail);
    let order_json = serde_json::json!({
        "product": order.product,
        "quantity": quantity,
        "address": address,
        "reference": gift.id,
    });
    let file = reqwest::multipart::Part::bytes(pdf.to_vec())
        .file_name("card.pdf")
        .mime_str("application/pdf")
        .map_err(ApiError::internal)?;
    let order_part = reqwest::multipart::Part::text(order_json.to_string())
        .mime_str("application/json")
        .map_err(ApiError::internal)?;
    let form = reqwest::multipart::Form::new().part("order", order_part).part("file", file);
    let response =
        state.http.post(format!("{printer}/v1/orders")).multipart(form).send().await.map_err(
            |e| unavailable(format!("the print provider did not answer: {}", e.without_url())),
        )?;
    let printer_status = response.status();
    let answer: serde_json::Value = match response.bytes().await {
        Ok(bytes) if bytes.len() <= MAX_PRINTER_ANSWER_BYTES => {
            serde_json::from_slice(&bytes).unwrap_or_default()
        }
        _ => serde_json::Value::Null,
    };
    if printer_status.is_client_error() {
        return Err(printer_refusal(printer_status, &answer));
    }
    let order_id = answer
        .get("order_id")
        .and_then(serde_json::Value::as_str)
        .filter(|_| printer_status.is_success());
    let Some(order_id) = order_id else {
        return Err(unavailable(format!("the print provider answered {printer_status}")));
    };
    let record = PrintOrderRecord {
        order_id: order_id.to_owned(),
        product: order.product,
        created_at: state.now(),
    };
    state.db.update(|db| {
        if let Some(gift) = db.gifts.get_mut(&id) {
            gift.print_orders.push(record);
        }
    })?;
    Ok(serde_json::json!({
        "order_id": order_id,
        "status": answer.get("status").cloned().unwrap_or_else(|| "accepted".into()),
        "price_cents": answer.get("price_cents").cloned().unwrap_or_default(),
        "currency": answer.get("currency").cloned().unwrap_or_else(|| "EUR".into()),
    }))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn address() -> serde_json::Value {
        json!({ "name": " Sacha Martin ", "line1": "12 rue des Lilas", "postal_code": "69003", "city": "Lyon" })
    }

    #[test]
    fn a_complete_address_is_trimmed_and_defaults_to_france() {
        let parsed = PostalAddress::from_json(&address()).expect("valid");
        assert_eq!(parsed.name, "Sacha Martin");
        assert_eq!(parsed.country, "FR");
        assert_eq!(parsed.line2, None);
        assert_eq!(
            serde_json::to_value(&parsed).expect("json"),
            json!({ "name": "Sacha Martin", "line1": "12 rue des Lilas", "postal_code": "69003", "city": "Lyon", "country": "FR" })
        );
        let mut with = address();
        with["line2"] = "Bâtiment B".into();
        with["country"] = " be ".into();
        let parsed = PostalAddress::from_json(&with).expect("valid");
        assert_eq!((parsed.line2.as_deref(), parsed.country.as_str()), (Some("Bâtiment B"), "BE"));
        with["line2"] = "  ".into();
        with["country"] = serde_json::Value::Null;
        let parsed = PostalAddress::from_json(&with).expect("valid");
        assert_eq!((parsed.line2, parsed.country.as_str()), (None, "FR"));
    }

    #[test]
    fn malformed_addresses_name_the_field() {
        let with = |key: &str, value: serde_json::Value| {
            let mut address = address();
            address[key] = value;
            address
        };
        let without = |key: &str| {
            let mut address = address();
            address.as_object_mut().expect("object").remove(key);
            address
        };
        let cases = [
            (json!("12 rue des Lilas, Lyon"), "address"),
            (json!({ "text": "12 rue des Lilas" }), "address.text"),
            (json!(null), "address"),
            (without("name"), "address.name"),
            (without("line1"), "address.line1"),
            (without("postal_code"), "address.postal_code"),
            (without("city"), "address.city"),
            (with("city", json!("   ")), "address.city"),
            (with("postal_code", json!(69003)), "address.postal_code"),
            (with("postal_code", json!("1".repeat(13))), "address.postal_code"),
            (with("name", json!("x".repeat(81))), "address.name"),
            (with("line2", json!("y".repeat(121))), "address.line2"),
            (with("line1", json!("a\u{0}b")), "address.line1"),
            (with("country", json!("FRA")), "address.country"),
            (with("country", json!("F1")), "address.country"),
            (with("country", json!("É")), "address.country"),
            (with("postcode", json!("69003")), "address.postcode"),
            (with("\u{7}", json!("x")), "address."),
        ];
        for (value, field) in cases {
            let error = PostalAddress::from_json(&value).expect_err("refused");
            assert_eq!((error.status, error.code), (StatusCode::BAD_REQUEST, "invalid_address"));
            assert_eq!(error.body()["error"]["field"], field, "{value}");
        }
        // Characters, not bytes, are counted.
        assert!(PostalAddress::from_json(&with("name", json!("é".repeat(80)))).is_ok());
    }

    #[test]
    fn printer_refusals_keep_the_printers_code() {
        let preflight = json!({ "error": { "code": "preflight_failed", "message": "2 error(s)", "findings": [{ "code": "BLEED" }] } });
        let error = printer_refusal(StatusCode::UNPROCESSABLE_ENTITY, &preflight);
        assert_eq!(
            (error.status, error.code),
            (StatusCode::UNPROCESSABLE_ENTITY, "preflight_failed")
        );
        assert_eq!(error.body()["error"]["findings"][0]["code"], "BLEED");
        assert_eq!(error.body()["error"]["printer_code"], "preflight_failed");

        let country = json!({ "error": { "code": "unsupported_country", "message": "this printer posts to FR only" } });
        let body = printer_refusal(StatusCode::UNPROCESSABLE_ENTITY, &country).body();
        assert_eq!(
            body,
            json!({ "error": {
                "code": "printer_rejected",
                "message": "the print provider refused the order (unsupported_country): this printer posts to FR only",
                "printer_status": 422, "printer_code": "unsupported_country",
                "printer_message": "this printer posts to FR only", "findings": [],
            } })
        );

        // A hostile or empty answer: no markup or control characters pass through.
        let hostile = json!({ "error": { "code": "<script>", "message": "a\u{0}b".repeat(500), "findings": "x" } });
        let body = printer_refusal(StatusCode::BAD_REQUEST, &hostile).body();
        assert_eq!(body["error"]["printer_code"], "http_400");
        assert_eq!(body["error"]["findings"], json!([]));
        let message = body["error"]["printer_message"].as_str().expect("message");
        assert!(message.chars().count() <= 300 && !message.contains('\u{0}'));
        let body = printer_refusal(StatusCode::CONFLICT, &serde_json::Value::Null).body();
        assert_eq!(
            (body["error"]["code"].as_str(), body["error"]["printer_code"].as_str()),
            (Some("printer_rejected"), Some("http_409"))
        );
        assert!(body["error"].get("printer_message").is_none());
    }
}
