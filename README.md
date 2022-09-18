# snekkja-rs
A lightweight static HTML/CSS/JS image gallery generator (Rust version).

`snekkja`'s operation and output are both simple: It reads through the
image files in the current directory and generates the appropriate
files (HTML, CSS, JS) to make the current directory a gallery.

## Build

As your target execution environment for this program is undoubtedly a web
server, you should probably build it for the `musl` target:

```sh
cargo build --release --target x86_64-unknown-linux-musl
```
and strip the debug symbols:
```
strip target/x86_64-unknown-linux-musl/release/snekkja
```

This is now a self-contained standalone binary that has all the info it
needs baked in. Install it in your web server user account's $PATH somewhere.

## Use

Place your image files in a directory accessible from the outside world, and
run:

```sh
snekkja
```

It will generate four files in that directory:
  * `index.html`
  * `snekkja.css`
  * `snekkja.js`
  * `data.js`
  * `prev.svg`
  * `next.svg`

which will present those image files in a gallery when you point your web
browser at that directory.

You have two options for limited customization:

```sh
snekkja -c
```
will generate a file `config.toml` in which some options can be set.
Deleting any of these values from the config file will cause the
default value to be used.

Also, creating a file `user.css` will allow you to customize CSS rules.

### Captions

Captions for images in the gallery can be specified in any combination of
the following three ways:

  * creating a file `captions.toml`

The keys are filenames, the values are the captions. So something like:

```toml
"foobar.jpg" = "An image of Dennis Ritchie holding the original FOOBAR."
"gsc_logo.png" = "The proposed standardized <em>Gay Space Communism</em> logo."
```

  * creating a file `captions.json`

Basically the previous file, but in JSON format.

```json
{
    "foobar.jpg": "An image of Dennis Ritche holding the original FOOBAR.",
    "gsc_logo.png": "The proposed standardized <em>Gay Space Communism</em> logo."
}
```

  * for a given image file, say `furry_riot.webp`, creating a file
    `furry_riot.html` with desired caption

Example contents of `furry_riot.html`:
```html
<p>
    Furry Riots, Los Angeles, 2024
</p>
<p>
    Weary of long-standing media stigmatization, Furries
    gather in downtown Los Angeles, don their cultural
    attire, and set fire to the headquarters of the
    <em>Los Angeles Times</em>.
</p>
```

**NOTE:** Caption text is added _verbatim_ to the HTML in the gallery
document, so markup certainly works. Be careful, though, because incorrect
or inappropriate markup definitely _can_ hose the appearance and function
of your page. `snekkja` makes no attempt to save you from yourself in this
regard. When in doubt, plain text will absolutely serve.

In the case that an image has a caption specified in multiple ways, captions
specified in individual HTML files will supercede those specified in
`captions.json` which will supercede those in `captions.toml`.

## Caveats

File names are handled internally as UTF-8 strings. I think this is okay
because they are going to be embedded in a UTF-8 Javascript file and used
as relative URLs. If you have a use case that requires non-UTF-8 filenames,
let me know, I guess, and I'll try to figure something out.