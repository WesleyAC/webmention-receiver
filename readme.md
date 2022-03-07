# `webmention-receiver`

`webmention-receiver` is a simple program that receives [webmentions](https://indieweb.org/Webmention), records them to a SQLite database, and allows viewing them via a webpage or RSS feed. It has no conception of an "account", and by default will accept webmentions for any domain. It is possible to configure it to only accept webmentions for a specific set of domains, if you'd prefer.

You are welcome to use the public instance, at [webmention.wesleyac.com](https://webmention.wesleyac.com/).

## Running it yourself

While you're welcome to use the [public instance](https://webmention.wesleyac.com), `webmention-receiver` is designed to be dead-easy to run yourself as well. It is a statically linked executable with no dependencies, which should run on any x86_64 Linux system. You can download it from the [releases page](https://github.com/WesleyAC/webmention-receiver). To run it:

* Download the most recent release onto your server.
* Make a file called [`config.toml`](/config.toml) file like the one in the repo, in the same place the executable you downloaded is. The only setting you need to set is `external_url`, which is the URL that the service will be accessible at (for instance, the public version is set to `https://webmention.wesleyac.com`).
* Run the `webmention-receiver` executable.

I would recommend using nginx or some other reverse proxy, and using certbot to get https certificates, although you don't have to if you don't want. I'd also recommend setting up the server so that it is started automatically, using something like systemd.

The main database is the `webmention-receiver.sqlite3` file. Whenever there's a database migration (which may happen when you switch to a new version of the server), a full copy of the database will be taken first, in a `webmention-receiver.migrationbackup.<timestamp>.<version>.sqlite3` file. If the migration completes successfully, it's fine to delete the `webmention-receiver.migrationbackup` files.

## Config

Here is a full list of the config options:

* `external_url` (required): The URL that this instance will be served at. There should be no trailing slash.
* `bind` (default: `127.0.0.1:28081`): The IP address and port to bind to.
* `allowed_domains` (default: unset): A list of domains that the server will accept webmentions for. Leave this unset to allow webmentions for any domain.
