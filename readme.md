# `webmention-receiver`

`webmention-receiver` is a simple program that receives [webmentions](https://indieweb.org/Webmention), records them to a SQLite database, and allows viewing them via a webpage or RSS feed. It has no conception of an "account", and by default will accept webmentions for any domain. It is possible to configure it to only accept webmentions for a specific set of domains, if you'd prefer.
