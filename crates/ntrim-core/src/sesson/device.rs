
#[derive(Debug, Clone, Default)]
pub struct Device {
    pub(crate) android_id: String,
    pub(crate) qimei: String,
    pub(crate) device_name: String,
    pub(crate) device_model: String,
    pub(crate) os_ver: String,
    pub(crate) vendor_os_name: String,
}