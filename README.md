# Development

Simply running the Rust code, e.g. `cargo run`, will start up a local server on port 3000 using axum. The HTML templates are found in `/templates` and are rendered using minijinja. Static files such as CSS, JavaScript, and images are found in `/public` and also get served from matching `/public/*` paths from the server.

If you add or change any page routes, you have to also change the `page_paths` array in the `build_static` function for the page to be rendered in static generation later.

# Static site generation and deployment

Running the Rust code with a `--build` argument, e.g. `cargo run -- --build`, will render the site to static files in the `/dist` directory. If you want to deploy to Cloudflare Pages, change the `name` value in `wrangler.toml` to a name of a project on your Cloudflare account, then run `npx wrangler pages deploy`.
