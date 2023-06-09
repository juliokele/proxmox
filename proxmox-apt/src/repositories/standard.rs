use std::fmt::Display;

use anyhow::{bail, Error};
use serde::{Deserialize, Serialize};

use crate::repositories::repository::{
    APTRepository, APTRepositoryFileType, APTRepositoryPackageType,
};

use proxmox_schema::api;

#[api(
    properties: {
        handle: {
            description: "Handle referencing a standard repository.",
            type: String,
        },
    },
)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
/// Reference to a standard repository and configuration status.
pub struct APTStandardRepository {
    /// Handle referencing a standard repository.
    pub handle: APTRepositoryHandle,

    /// Configuration status of the associated repository.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<bool>,

    /// Display name of the repository.
    pub name: String,

    /// Description of the repository.
    pub description: String,
}

#[api]
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
/// Handles for Proxmox repositories.
pub enum APTRepositoryHandle {
    /// The enterprise repository for production use.
    Enterprise,
    /// The repository that can be used without subscription.
    NoSubscription,
    /// The test repository.
    Test,
    /// Ceph Quincy repository.
    CephQuincy,
    /// Ceph Quincy test repository.
    CephQuincyTest,
    /// Ceph Pacific repository.
    CephPacific,
    /// Ceph Pacific test repository.
    CephPacificTest,
    /// Ceph Octoput repository.
    CephOctopus,
    /// Ceph Octoput test repository.
    CephOctopusTest,
}

impl From<APTRepositoryHandle> for APTStandardRepository {
    fn from(handle: APTRepositoryHandle) -> Self {
        APTStandardRepository {
            handle,
            status: None,
            name: handle.name(),
            description: handle.description(),
        }
    }
}

impl TryFrom<&str> for APTRepositoryHandle {
    type Error = Error;

    fn try_from(string: &str) -> Result<Self, Error> {
        match string {
            "enterprise" => Ok(APTRepositoryHandle::Enterprise),
            "no-subscription" => Ok(APTRepositoryHandle::NoSubscription),
            "test" => Ok(APTRepositoryHandle::Test),
            "ceph-quincy" => Ok(APTRepositoryHandle::CephQuincy),
            "ceph-quincy-test" => Ok(APTRepositoryHandle::CephQuincyTest),
            "ceph-pacific" => Ok(APTRepositoryHandle::CephPacific),
            "ceph-pacific-test" => Ok(APTRepositoryHandle::CephPacificTest),
            "ceph-octopus" => Ok(APTRepositoryHandle::CephOctopus),
            "ceph-octopus-test" => Ok(APTRepositoryHandle::CephOctopusTest),
            _ => bail!("unknown repository handle '{}'", string),
        }
    }
}

impl Display for APTRepositoryHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            APTRepositoryHandle::Enterprise => write!(f, "enterprise"),
            APTRepositoryHandle::NoSubscription => write!(f, "no-subscription"),
            APTRepositoryHandle::Test => write!(f, "test"),
            APTRepositoryHandle::CephQuincy => write!(f, "ceph-quincy"),
            APTRepositoryHandle::CephQuincyTest => write!(f, "ceph-quincy-test"),
            APTRepositoryHandle::CephPacific => write!(f, "ceph-pacific"),
            APTRepositoryHandle::CephPacificTest => write!(f, "ceph-pacific-test"),
            APTRepositoryHandle::CephOctopus => write!(f, "ceph-octopus"),
            APTRepositoryHandle::CephOctopusTest => write!(f, "ceph-octopus-test"),
        }
    }
}

impl APTRepositoryHandle {
    /// Get the description for the repository.
    pub fn description(self) -> String {
        match self {
            APTRepositoryHandle::Enterprise => {
                "This is the default, stable, and recommended repository, available for all \
                Proxmox subscription users."
            }
            APTRepositoryHandle::NoSubscription => {
                "This is the recommended repository for testing and non-production use. \
                Its packages are not as heavily tested and validated as the production ready \
                enterprise repository. You don't need a subscription key to access this repository."
            }
            APTRepositoryHandle::Test => {
                "This repository contains the latest packages and is primarily used for test labs \
                and by developers to test new features."
            }
            APTRepositoryHandle::CephQuincy => {
                "This repository holds the main Proxmox Ceph Quincy packages."
            }
            APTRepositoryHandle::CephQuincyTest => {
                "This repository contains the Ceph Quincy packages before they are moved to the \
                main repository."
            }
            APTRepositoryHandle::CephPacific => {
                "This repository holds the main Proxmox Ceph Pacific packages."
            }
            APTRepositoryHandle::CephPacificTest => {
                "This repository contains the Ceph Pacific packages before they are moved to the \
                main repository."
            }
            APTRepositoryHandle::CephOctopus => {
                "This repository holds the main Proxmox Ceph Octopus packages."
            }
            APTRepositoryHandle::CephOctopusTest => {
                "This repository contains the Ceph Octopus packages before they are moved to the \
                main repository."
            }
        }
        .to_string()
    }

    /// Get the display name of the repository.
    pub fn name(self) -> String {
        match self {
            APTRepositoryHandle::Enterprise => "Enterprise",
            APTRepositoryHandle::NoSubscription => "No-Subscription",
            APTRepositoryHandle::Test => "Test",
            APTRepositoryHandle::CephQuincy => "Ceph Quincy",
            APTRepositoryHandle::CephQuincyTest => "Ceph Quincy Test",
            APTRepositoryHandle::CephPacific => "Ceph Pacific",
            APTRepositoryHandle::CephPacificTest => "Ceph Pacific Test",
            APTRepositoryHandle::CephOctopus => "Ceph Octopus",
            APTRepositoryHandle::CephOctopusTest => "Ceph Octopus Test",
        }
        .to_string()
    }

    /// Get the standard file path for the repository referenced by the handle.
    pub fn path(self, product: &str) -> String {
        match self {
            APTRepositoryHandle::Enterprise => {
                format!("/etc/apt/sources.list.d/{}-enterprise.list", product)
            }
            APTRepositoryHandle::NoSubscription => "/etc/apt/sources.list".to_string(),
            APTRepositoryHandle::Test => "/etc/apt/sources.list".to_string(),
            APTRepositoryHandle::CephQuincy => "/etc/apt/sources.list.d/ceph.list".to_string(),
            APTRepositoryHandle::CephQuincyTest => "/etc/apt/sources.list.d/ceph.list".to_string(),
            APTRepositoryHandle::CephPacific => "/etc/apt/sources.list.d/ceph.list".to_string(),
            APTRepositoryHandle::CephPacificTest => "/etc/apt/sources.list.d/ceph.list".to_string(),
            APTRepositoryHandle::CephOctopus => "/etc/apt/sources.list.d/ceph.list".to_string(),
            APTRepositoryHandle::CephOctopusTest => "/etc/apt/sources.list.d/ceph.list".to_string(),
        }
    }

    /// Get package type, possible URIs and the component associated with the handle.
    ///
    /// The first URI is the preferred one.
    pub fn info(self, product: &str) -> (APTRepositoryPackageType, Vec<String>, String) {
        match self {
            APTRepositoryHandle::Enterprise => (
                APTRepositoryPackageType::Deb,
                match product {
                    "pve" => vec![
                        "https://enterprise.proxmox.com/debian/pve".to_string(),
                        "https://enterprise.proxmox.com/debian".to_string(),
                    ],
                    _ => vec![format!("https://enterprise.proxmox.com/debian/{}", product)],
                },
                format!("{}-enterprise", product),
            ),
            APTRepositoryHandle::NoSubscription => (
                APTRepositoryPackageType::Deb,
                match product {
                    "pve" => vec![
                        "http://download.proxmox.com/debian/pve".to_string(),
                        "http://download.proxmox.com/debian".to_string(),
                    ],
                    _ => vec![format!("http://download.proxmox.com/debian/{}", product)],
                },
                format!("{}-no-subscription", product),
            ),
            APTRepositoryHandle::Test => (
                APTRepositoryPackageType::Deb,
                match product {
                    "pve" => vec![
                        "http://download.proxmox.com/debian/pve".to_string(),
                        "http://download.proxmox.com/debian".to_string(),
                    ],
                    _ => vec![format!("http://download.proxmox.com/debian/{}", product)],
                },
                format!("{}test", product),
            ),
            APTRepositoryHandle::CephQuincy => (
                APTRepositoryPackageType::Deb,
                vec!["http://download.proxmox.com/debian/ceph-quincy".to_string()],
                "main".to_string(),
            ),
            APTRepositoryHandle::CephQuincyTest => (
                APTRepositoryPackageType::Deb,
                vec!["http://download.proxmox.com/debian/ceph-quincy".to_string()],
                "test".to_string(),
            ),
            APTRepositoryHandle::CephPacific => (
                APTRepositoryPackageType::Deb,
                vec!["http://download.proxmox.com/debian/ceph-pacific".to_string()],
                "main".to_string(),
            ),
            APTRepositoryHandle::CephPacificTest => (
                APTRepositoryPackageType::Deb,
                vec!["http://download.proxmox.com/debian/ceph-pacific".to_string()],
                "test".to_string(),
            ),
            APTRepositoryHandle::CephOctopus => (
                APTRepositoryPackageType::Deb,
                vec!["http://download.proxmox.com/debian/ceph-octopus".to_string()],
                "main".to_string(),
            ),
            APTRepositoryHandle::CephOctopusTest => (
                APTRepositoryPackageType::Deb,
                vec!["http://download.proxmox.com/debian/ceph-octopus".to_string()],
                "test".to_string(),
            ),
        }
    }

    /// Get the standard repository referenced by the handle.
    ///
    /// An URI in the result is not '/'-terminated (under the assumption that no valid
    /// product name is).
    pub fn to_repository(self, product: &str, suite: &str) -> APTRepository {
        let (package_type, uris, component) = self.info(product);

        APTRepository {
            types: vec![package_type],
            uris: vec![uris.into_iter().next().unwrap()],
            suites: vec![suite.to_string()],
            components: vec![component],
            options: vec![],
            comment: String::new(),
            file_type: APTRepositoryFileType::List,
            enabled: true,
        }
    }
}
