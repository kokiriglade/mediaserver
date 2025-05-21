use askama::Template;

#[derive(Template)]
#[template(path = "directory_view.html")]
pub struct DirectoryView<'a> {
    pub current_directory: &'a str,
    pub total_items: &'a str,
    pub parent_dir_href: &'a str,
    pub individual_listings: &'a Vec<IndividualListing>,
}

#[derive(Template)]
#[template(path = "individual_listing.html")]
pub struct IndividualListing {
    pub emoji: String,
    pub timestamp: String,
    pub file_href: String,
    pub file_name: String,
    pub byte_size: String,
    pub is_directory: bool,
}
