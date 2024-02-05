use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Document {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Head")]
    pub head: Head,
    #[serde(rename = "General")]
    pub general: General,
    #[serde(rename = "NetworkInformation")]
    pub network_information: NetworkInformation,
    #[serde(rename = "ServerConfigurationSettings")]
    pub server_configuration_settings: ServerConfigurationSettings,
    #[serde(rename = "CertificateInformation")]
    pub certificate_information: CertificateInformation,
    #[serde(rename = "Components")]
    pub components: Components,
    #[serde(rename = "CameraRecordingStorage")]
    pub camera_recording_storage: CameraRecordingStorage,
    #[serde(rename = "UpdateService")]
    pub update_service: UpdateService,
    #[serde(rename = "LicenseInfo")]
    pub license_info: LicenseInfo,
    #[serde(rename = "SystemInfo")]
    pub system_info: SystemInfo,
    #[serde(rename = "Onboarding")]
    pub onboarding: Onboarding,
    #[serde(rename = "SystemSynchronization")]
    pub system_synchronization: SystemSynchronization,
    #[serde(rename = "ProxySettings")]
    pub proxy_settings: ProxySettings,
    #[serde(rename = "Registry")]
    pub registry: Registry,
    #[serde(rename = "Modules")]
    pub modules: Modules,
    #[serde(rename = "SRA")]
    pub sra: Sra,
    #[serde(rename = "FeatureToggles")]
    pub feature_toggles: FeatureToggles,
    #[serde(rename = "CameraMetadataSettings")]
    pub camera_metadata_settings: CameraMetadataSettings,
    #[serde(rename = "CameraSettings")]
    pub camera_settings: CameraSettings,
    #[serde(rename = "VideoAndAudioSettings")]
    pub video_and_audio_settings: VideoAndAudioSettings,
    #[serde(rename = "CameraRecordingSettings")]
    pub camera_recording_settings: CameraRecordingSettings,
    // #[serde(rename = "Rules")]
    // pub rules: Rules,
    #[serde(rename = "Schedules")]
    pub schedules: Schedules,
    #[serde(rename = "Views")]
    pub views: Views,
    #[serde(rename = "Identities")]
    pub identities: Identities,
    #[serde(rename = "Privileges")]
    pub privileges: Privileges,
    #[serde(rename = "DeviceSettings")]
    pub device_settings: DeviceSettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Head {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Subtitle")]
    pub subtitle: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct General {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: Vec<GeneralSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneralSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkInformation {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: Vec<NetworkInformationSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkInformationSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfigurationSettings {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: Vec<ServerConfigurationSettingsSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfigurationSettingsSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CertificateInformation {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: Vec<CertificateInformationSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CertificateInformationSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Values")]
    pub values: CertificateInformationSettingValues,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CertificateInformationSettingValues {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Value")]
    pub value: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Components {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Component")]
    pub component: Vec<Component>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Component {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "ContentDocument")]
    pub content_document: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "PluginId")]
    pub plugin_id: String,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "Status")]
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraRecordingStorage {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Disc")]
    pub disc: Vec<Disc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Disc {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: Vec<DiscSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateService {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: Vec<UpdateServiceSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateServiceSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseInfo {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: Vec<LicenseInfoSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseInfoSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: Vec<SystemInfoSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfoSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Onboarding {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: OnboardingSetting,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OnboardingSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemSynchronization {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: SystemSynchronizationSetting,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemSynchronizationSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxySettings {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: Vec<ProxySettingsSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProxySettingsSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Registry {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: RegistrySetting,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrySetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Modules {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: Vec<ModulesSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModulesSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sra {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: SraSetting,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SraSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureToggles {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: FeatureTogglesSetting,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureTogglesSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraMetadataSettings {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CameraMetadataSetting")]
    pub camera_metadata_setting: Vec<CameraMetadataSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraMetadataSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: Vec<CameraMetadataSettingSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraMetadataSettingSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Value")]
    pub value: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Values")]
    pub values: Option<CameraMetadataSettingSettingValues>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraMetadataSettingSettingValues {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Value")]
    pub value: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraSettings {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CameraSetting")]
    pub camera_setting: Vec<CameraSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: Vec<CameraSettingSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraSettingSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoAndAudioSettings {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "VideoAndAudioSetting")]
    pub video_and_audio_setting: Vec<VideoAndAudioSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoAndAudioSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: Vec<VideoAndAudioSettingSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoAndAudioSettingSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Values")]
    pub values: Option<VideoAndAudioSettingSettingValues>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoAndAudioSettingSettingValues {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Value")]
    pub value: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraRecordingSettings {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "CameraRecordingSetting")]
    pub camera_recording_setting: Vec<CameraRecordingSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraRecordingSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: Vec<CameraRecordingSettingSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraRecordingSettingSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Values")]
    pub values: Option<CameraRecordingSettingSettingValues>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraRecordingSettingSettingValues {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rules {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Rule")]
    pub rule: Vec<Rule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rule {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: Vec<RuleSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Values")]
    pub values: Option<RuleSettingValues>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleSettingValues {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Value")]
    pub value: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Schedules {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Schedule")]
    pub schedule: Vec<Schedule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Schedule {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: Vec<ScheduleSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduleSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Value")]
    pub value: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Values")]
    pub values: Option<ScheduleSettingValues>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduleSettingValues {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Value")]
    pub value: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Views {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "View")]
    pub view: Vec<View>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct View {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: Vec<ViewSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ViewSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Identities {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Identity")]
    pub identity: Vec<Identity>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Identity {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: Vec<IdentitySetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentitySetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Privileges {}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceSettings {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "DeviceSetting")]
    pub device_setting: DeviceSetting,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Setting")]
    pub setting: Vec<DeviceSettingSetting>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceSettingSetting {
    #[serde(rename = "$text")]
    pub text: Option<String>,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Value")]
    pub value: String,
}
