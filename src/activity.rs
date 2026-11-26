use crate::night_review;
use serde::Serialize;
use serde_json::{Value, json};
use std::process::Command;

#[derive(Serialize)]
struct Event {
    unit: String,
    timestamp_ms: u64,
    status: &'static str,
    message: String,
}

fn events(output: &[u8]) -> Result<Vec<Event>, String> {
    let text =
        std::str::from_utf8(output).map_err(|error| format!("Invalid journal UTF-8: {error}"))?;
    let mut result = Vec::new();
    for line in text.lines() {
        let value: Value = serde_json::from_str(line)
            .map_err(|error| format!("Invalid journal entry: {error}"))?;
        let get = |key: &str| value.get(key).and_then(Value::as_str);
        let unit = get("USER_UNIT")
            .or_else(|| get("_SYSTEMD_USER_UNIT"))
            .ok_or("Journal unit missing")?;
        if !matches!(unit, "klm-research.service" | "klm-backup.service") {
            return Err(format!("Unexpected journal unit: {unit}"));
        }
        let timestamp: u64 = get("__REALTIME_TIMESTAMP")
            .ok_or("Journal timestamp missing")?
            .parse()
            .map_err(|_| "Invalid journal timestamp")?;
        let priority: u8 = get("PRIORITY")
            .ok_or("Journal priority missing")?
            .parse()
            .map_err(|_| "Invalid journal priority")?;
        let message = get("MESSAGE").ok_or("Journal message missing or not text")?;
        let status = if priority <= 3 {
            "error"
        } else if get("MESSAGE_ID") == Some("39f53479d3a045ac8e11786248231fbf") {
            "finished"
        } else if get("MESSAGE_ID") == Some("7d4958e842da4a758f6c1cdc7b36dcc5") {
            "started"
        } else if priority == 4 {
            "warning"
        } else {
            "info"
        };
        result.push(Event {
            unit: unit.into(),
            timestamp_ms: timestamp / 1000,
            status,
            message: message.into(),
        });
    }
    result.reverse();
    Ok(result)
}

fn services(output: &[u8]) -> Result<Vec<Value>, String> {
    let text =
        std::str::from_utf8(output).map_err(|error| format!("Invalid systemd output: {error}"))?;
    let mut result = Vec::new();
    for block in text.trim().split("\n\n") {
        let fields: std::collections::BTreeMap<_, _> = block
            .lines()
            .filter_map(|line| line.split_once('='))
            .collect();
        let id = fields.get("Id").ok_or("Missing service ID")?;
        if !matches!(*id, "klm-research.service" | "klm-backup.service")
            || result.iter().any(|entry: &Value| entry["id"] == *id)
        {
            return Err(format!("Invalid or duplicate service: {id}"));
        }
        result.push(json!({
            "id": id,
            "active": fields.get("ActiveState").ok_or("Missing service state")?,
            "substate": fields.get("SubState").ok_or("Missing service substate")?,
            "result": fields.get("Result").ok_or("Missing service result")?,
        }));
    }
    if result.len() != 2 {
        return Err("Both KLM service states are required".into());
    }
    Ok(result)
}

fn command(program: &str, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("Cannot run {program}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "{program} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}

pub(crate) fn read() -> Result<Value, String> {
    let status = command(
        "systemctl",
        &[
            "--user",
            "show",
            "klm-research.service",
            "klm-backup.service",
            "--property=Id,ActiveState,SubState,Result",
            "--no-pager",
        ],
    )?;
    let journal = command(
        "journalctl",
        &[
            "--user",
            "--no-pager",
            "--output=json",
            "--lines=50",
            "--unit=klm-research.service",
            "--unit=klm-backup.service",
        ],
    )?;
    Ok(json!({
        "mode": if night_review::is_night(chrono::Local::now()) { "night" } else { "day" },
        "services": services(&status)?,
        "events": events(&journal)?,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn journal_reports_actual_status_and_rejects_bad_entries() {
        let text = br#"{"USER_UNIT":"klm-backup.service","MESSAGE":"Finished backup","MESSAGE_ID":"39f53479d3a045ac8e11786248231fbf","PRIORITY":"6","__REALTIME_TIMESTAMP":"1234000"}
{"_SYSTEMD_USER_UNIT":"klm-research.service","MESSAGE":"network failed","PRIORITY":"3","__REALTIME_TIMESTAMP":"1235000"}"#;
        let events = events(text).unwrap();
        assert_eq!(events[0].status, "error");
        assert_eq!(events[1].status, "finished");
        assert_eq!(events[0].timestamp_ms, 1235);
        assert!(super::events(br#"{"MESSAGE":"missing fields"}"#).is_err());
    }

    #[test]
    fn service_states_are_explicit() {
        let text = b"Id=klm-research.service\nActiveState=active\nSubState=running\nResult=success\n\nId=klm-backup.service\nActiveState=failed\nSubState=failed\nResult=exit-code\n";
        let states = services(text).unwrap();
        assert_eq!(states[0]["active"], "active");
        assert_eq!(states[1]["result"], "exit-code");
        assert!(services(b"Id=klm-research.service\n").is_err());
    }
}
