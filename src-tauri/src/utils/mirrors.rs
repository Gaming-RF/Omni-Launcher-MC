use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Mirror {
    Official,
    BMCLAPI,
    MCBBS,
}

impl Mirror {
    pub fn id(&self) -> &str {
        match self {
            Mirror::Official => "official",
            Mirror::BMCLAPI => "bmclapi",
            Mirror::MCBBS => "mcbbs",
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Mirror::Official => "Official (Mojang)",
            Mirror::BMCLAPI => "BMCLAPI (Asia)",
            Mirror::MCBBS => "MCBBS (China)",
        }
    }

    pub fn base_url(&self) -> &str {
        match self {
            Mirror::Official => "",
            Mirror::BMCLAPI => "https://bmclapi2.bangbang93.com",
            Mirror::MCBBS => "https://download.mcbbs.net",
        }
    }

    pub fn all() -> Vec<Mirror> {
        vec![Mirror::Official, Mirror::BMCLAPI, Mirror::MCBBS]
    }

    /// Parse a mirror from its string id. Returns `None` for unrecognized ids.
    pub fn from_id(id: &str) -> Option<Mirror> {
        match id {
            "official" => Some(Mirror::Official),
            "bmclapi" => Some(Mirror::BMCLAPI),
            "mcbbs" => Some(Mirror::MCBBS),
            _ => None,
        }
    }
}

/// Resolve a Mojang URL to the mirror equivalent.
///
/// For `Official` mirrors this is a no-op. For BMCLAPI / MCBBS the function
/// rewrites the well-known Mojang CDN domains to the mirror's equivalents.
pub fn resolve_url(original_url: &str, mirror: &Mirror) -> String {
    match mirror {
        Mirror::Official => original_url.to_string(),
        Mirror::BMCLAPI | Mirror::MCBBS => {
            let base = mirror.base_url();
            let url = original_url
                .replace(
                    "https://launchermeta.mojang.com",
                    &format!("{}/v1/packages", base),
                )
                .replace(
                    "https://piston-meta.mojang.com",
                    &format!("{}/v1/packages", base),
                )
                .replace(
                    "https://piston-data.mojang.com",
                    &format!("{}/version", base),
                )
                .replace(
                    "https://libraries.minecraft.net",
                    &format!("{}/maven", base),
                )
                .replace(
                    "https://resources.download.minecraft.net",
                    &format!("{}/assets", base),
                );
            url
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_url_official() {
        let url = "https://launchermeta.mojang.com/foo";
        assert_eq!(resolve_url(url, &Mirror::Official), url);
    }

    #[test]
    fn test_resolve_url_bmclapi() {
        let url = "https://launchermeta.mojang.com/v1/manifest/1.20";
        assert_eq!(
            resolve_url(url, &Mirror::BMCLAPI),
            "https://bmclapi2.bangbang93.com/v1/packages/v1/manifest/1.20"
        );
    }

    #[test]
    fn test_resolve_url_libraries() {
        let url = "https://libraries.minecraft.net/com/mojang/foo.jar";
        assert_eq!(
            resolve_url(url, &Mirror::MCBBS),
            "https://download.mcbbs.net/maven/com/mojang/foo.jar"
        );
    }

    #[test]
    fn test_from_id() {
        assert_eq!(Mirror::from_id("bmclapi"), Some(Mirror::BMCLAPI));
        assert_eq!(Mirror::from_id("unknown"), None);
    }

    #[test]
    fn test_all_ids() {
        let ids: Vec<&str> = Mirror::all().iter().map(|m| m.id()).collect();
        assert_eq!(ids, vec!["official", "bmclapi", "mcbbs"]);
    }
}
