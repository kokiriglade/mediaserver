# mediaserver

A very configurable personal file uploading service.

## Build
* [Install Rust](https://www.rust-lang.org/tools/install)
* Clone the repository
* Run `cargo build --release`

## Setup

By default the server will generate a configuration file for you. Here's the
documentation for said file (`mediaserver.toml`):

```toml
[web_server]
host = "127.0.0.1"
port = 3000
listen_url = "http://localhost:3000/"
# feel free to change it. i won't be offended :^)
redirect_index_to = "http://github.com/kokiriglade/mediaserver"

[file_listing_render.emoji]
directory = "📂"
unknown = "❓" # for file extensions we don't know

# mapping of file extensions to emoji
[file_listing_render.emoji.file_extensions]
png = "🖼️"

[storage]
# the default namespace to use if one isn't specified.
# for example, if `f` has a file uploaded called `p.png`, we can access it
# using either:
# - http://localhost:3000/f/p.png
# or
# - http://localhost:3000/p.png
default_namespace_fs_path = "ferris"
max_file_size_bytes = 104857600
uploads_directory = "uploads"

[namespaces.f]
# so files will be stored in `uploads/ferris`, but accessible at `example.com/f/`
file_system_path = "ferris"
key = "a_secure_authentication_key_goes_here"

[namespaces.f.file_listing]
show = false
# produces a nicer HTML output but may be slower (likely not noticeable though)
use_fancy_renderer = true

# the file name generator to use for a namespace...
[namespaces.f.file_name_generator]
# can be either "random" or "uuid"
type = "random"

# these are only applied when type is "random"
length = 12 # initial target length
max_attempts_before_grow = 32 # if we fail to generate a unique file name after 
                              # 32 tries, we bump the target length by 1
```

## Usage

mediaserver was made primarily with ShareX support in mind. Here's a config you
can copy:

![A screenshot of the ShareX "Custom uploader settings" tab. The method has been set to "PUT", the body to "Form data (multipart/form-data), and "namespace" and "auth_key" set in the form body. The "file form name" option is set to "file", and the "URL" option is set to "{json:link}".](https://i.kokirigla.de/k/83b1a3d8-1645-4192-b9f4-d8e35280b5c2.png)