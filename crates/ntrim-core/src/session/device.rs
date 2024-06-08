use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct Device {
    pub android_id: String,
    pub qimei: String,
    pub device_name: String,
    pub brand: String,
    pub code: String,
    pub os_name: String,
    pub os_ver: String,
    pub vendor_os_name: String,
    pub fingerprint: Arc<Vec<u8>>,
    pub apn_name: String,
    pub apn: String,
    pub mac_address: String,
    pub os_type: String
}

impl Device {
    pub fn new(
        android_id: String,
        qimei: String,
        device_name: String,
        brand: String,
        os_ver: String,
        vendor_os_name: String,
        fingerprint: Vec<u8>,
        code: String,
        os_name: String
    ) -> Self {
        Self {
            android_id,
            qimei,
            device_name,
            brand,
            os_ver,
            vendor_os_name,
            fingerprint: Arc::new(fingerprint),
            code, os_name,
            apn_name: "".to_string(),
            apn: "wifi".to_string(),
            mac_address: "00:00:00:00:00:00".to_string(),
            os_type: "android".to_string()
        }
    }
}