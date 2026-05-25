// View modules

pub mod resources_pane;
pub mod details_pane;
pub mod logs_pane;
pub mod help_view;
pub mod command_palette;
pub mod search_view;
pub mod splash_view;

pub use resources_pane::ResourcesPane;
pub use details_pane::DetailsPane;
pub use logs_pane::LogsPane;
pub use help_view::HelpView;
#[allow(unused_imports)]
pub use command_palette::CommandPalette;
#[allow(unused_imports)]
pub use search_view::SearchView;
pub use splash_view::SplashView;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    Splash,
    Main,
    Help,
    // Future views (kept for forward compatibility)
    #[allow(dead_code)]
    NewDeployment,
    #[allow(dead_code)]
    Search(String),
    #[allow(dead_code)]
    CommandPalette,
}
