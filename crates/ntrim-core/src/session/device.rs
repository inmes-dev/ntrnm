use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct Device {
    pub android_id: String,
    pub qimei: String,
    pub device_name: String,
    pub device_model: String,
    pub os_ver: String,
    pub vendor_os_name: String,
    pub fingerprint: Arc<Vec<u8>>,
}

impl Device {
    pub fn new(
        android_id: String,
        qimei: String,
        device_name: String,
        device_model: String,
        os_ver: String,
        vendor_os_name: String,
        fingerprint: Vec<u8>,
    ) -> Self {
        Self {
            android_id,
            qimei,
            device_name,
            device_model,
            os_ver,
            vendor_os_name,
            fingerprint: Arc::new(fingerprint),
        }
    }
}