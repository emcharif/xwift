use std::fs;

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};

pub struct Sample {
    pub power_w: Option<i32>,
    pub cadence_rpm: Option<u32>,
    pub hr_bpm: Option<u16>,
    pub speed_kmh: Option<f64>,
}

const DESCRIPTION: &str =
    "Ridden with xwift, a home-built Rust trainer app: ERG workouts over Bluetooth, no subscription needed.";

/// Writes rides/<name>_<timestamp>.tcx and returns the path.
pub fn write_tcx(name: &str, start: DateTime<Utc>, samples: &[Sample]) -> Result<String> {
    fs::create_dir_all("rides")?;
    let slug: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let path = format!("rides/{}_{}.tcx", slug, start.format("%Y%m%d_%H%M"));

    let mut dist = 0.0_f64;
    let mut points = String::new();
    let mut hr_sum = 0u64;
    let mut hr_n = 0u64;
    let mut hr_max = 0u16;

    for (i, s) in samples.iter().enumerate() {
        dist += s.speed_kmh.unwrap_or(0.0) / 3.6; // 1 sample per second
        let t = (start + Duration::seconds(i as i64)).format("%Y-%m-%dT%H:%M:%SZ");
        points.push_str(&format!("<Trackpoint><Time>{t}</Time><DistanceMeters>{dist:.1}</DistanceMeters>"));
        if let Some(h) = s.hr_bpm {
            points.push_str(&format!("<HeartRateBpm><Value>{h}</Value></HeartRateBpm>"));
            hr_sum += h as u64;
            hr_n += 1;
            hr_max = hr_max.max(h);
        }
        if let Some(c) = s.cadence_rpm {
            points.push_str(&format!("<Cadence>{c}</Cadence>"));
        }
        if let Some(w) = s.power_w {
            points.push_str(&format!(
                "<Extensions><TPX xmlns=\"http://www.garmin.com/xmlschemas/ActivityExtension/v2\"><Watts>{}</Watts></TPX></Extensions>",
                w.max(0)
            ));
        }
        points.push_str("</Trackpoint>\n");
    }

    let hr_summary = if hr_n > 0 {
        format!(
            "<AverageHeartRateBpm><Value>{}</Value></AverageHeartRateBpm><MaximumHeartRateBpm><Value>{}</Value></MaximumHeartRateBpm>",
            hr_sum / hr_n,
            hr_max
        )
    } else {
        String::new()
    };

    let start_iso = start.format("%Y-%m-%dT%H:%M:%SZ");
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<TrainingCenterDatabase xmlns="http://www.garmin.com/xmlschemas/TrainingCenterDatabase/v2">
<Activities>
<Activity Sport="Biking">
<Id>{start_iso}</Id>
<Lap StartTime="{start_iso}">
<TotalTimeSeconds>{secs}</TotalTimeSeconds>
<DistanceMeters>{dist:.1}</DistanceMeters>
<Calories>0</Calories>
{hr_summary}
<Intensity>Active</Intensity>
<TriggerMethod>Manual</TriggerMethod>
<Track>
{points}</Track>
</Lap>
<Notes>{name}: {DESCRIPTION}</Notes>
</Activity>
</Activities>
</TrainingCenterDatabase>
"#,
        secs = samples.len(),
    );

    fs::write(&path, xml)?;
    Ok(path)
}