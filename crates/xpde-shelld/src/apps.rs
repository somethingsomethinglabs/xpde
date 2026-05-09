pub fn list_apps() -> Vec<String> {
    xpde_desktop::list_apps().into_iter().map(|a| a.name).collect()
}
