use serde_json::Value;

use crate::filter::EntityDescriptor;

pub fn map_descriptor_to_device_type(desc: &EntityDescriptor) -> Option<String> {
    let domain = desc.domain.as_str();
    let attrs = &desc.attributes;
    match domain {
        "automation" => Some("OnOffPlugInUnit".into()),
        "button" => Some("OnOffPlugInUnit".into()),
        "binary_sensor" => Some(map_binary_sensor(attrs)),
        "climate" => Some("Thermostat".into()),
        "cover" => Some("WindowCovering".into()),
        "fan" => Some("Fan".into()),
        "humidifier" => Some("OnOffPlugInUnit".into()),
        "input_boolean" => Some("OnOffPlugInUnit".into()),
        "input_button" => Some("OnOffPlugInUnit".into()),
        "light" => Some(map_light(attrs)),
        "lock" => Some("DoorLock".into()),
        "media_player" => Some("Speaker".into()),
        "scene" => Some("OnOffPlugInUnit".into()),
        "script" => Some("OnOffPlugInUnit".into()),
        "sensor" => map_sensor(attrs),
        "switch" => Some("OnOffPlugInUnit".into()),
        "vacuum" => Some("RoboticVacuumCleaner".into()),
        _ => None,
    }
}

fn map_binary_sensor(attrs: &Value) -> String {
    match attrs.get("device_class").and_then(|v| v.as_str()) {
        Some("opening") | Some("door") | Some("window") => "ContactSensor".into(),
        Some("motion") | Some("occupancy") => "OccupancySensor".into(),
        Some("moisture") | Some("water") | Some("leak") => "WaterLeakDetector".into(),
        _ => "OnOffSensor".into(),
    }
}

fn map_light(attrs: &Value) -> String {
    let modes = attrs
        .get("supported_color_modes")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if modes.iter().any(|m| matches!(*m, "xy" | "hs" | "rgb" | "rgbw" | "rgbww")) {
        return "ExtendedColorLight".into();
    }
    if modes.iter().any(|m| *m == "color_temp") {
        return "ColorTemperatureLight".into();
    }
    if modes.iter().any(|m| *m == "brightness") {
        return "DimmableLight".into();
    }

    "OnOffLight".into()
}

fn map_sensor(attrs: &Value) -> Option<String> {
    match attrs.get("device_class").and_then(|v| v.as_str()) {
        Some("temperature") => Some("TemperatureSensor".into()),
        Some("humidity") => Some("HumiditySensor".into()),
        Some("illuminance") => Some("IlluminanceSensor".into()),
        _ => None,
    }
}
