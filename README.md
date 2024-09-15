# GAT - GlazeWM Alternating Tiler, written in Rust.
### Eh, I'm done with this.
-----

### Why?
Really, there was no reason why other than exploring async and websockets using Rust. I haven't been messing around with programming
too much lately, so I figured that this would be a good starting point. Yes, it is shoddily-written and unnecessary, but why not?
And yes, this was initially based on the same premise as [Burgr033's work using Python, which can be found here.](https://github.com/burgr033/GlazeWM-autotiling-python)

-----
### What about-?
Yes, I know, you could use pytools to compile [the original script by Burgr033](https://github.com/burgr033/GlazeWM-autotiling-python) and run it as an executable, but I personally hate that approach and think that the packaging for Rust is superior in this regard.

For proper inspiration, look up [jonhoo's OBS-DO project](https://github.com/jonhoo/obs-do) along with some of his other work.

-----
### Okay, so how would I set this up to build from scratch?
Rust@Nightly-GNU:

```Rust
rustup install nightly-gnu
rustup default nightly-gnu
```

Open the project folder in a terminal,
then type `cargo b -r` to build a release binary, which will be located in `gat-gwm\target\release\gat-gwm.exe`.

You can also build a release binary with no console by specifying the `no_console` feature like so: `cargo b -r --features=no_console`

If all of that was too much, download the pre-made binary.

From here, you can add the executable to a script, bind it to a key in GWM, or just have it autostart in GWM V3 by adding it to the startup commands list:
```yaml
general:
  startup_commands: [
    ...
    'shell-exec <GATLocation>/gat-gwm.exe'
  ]
```

-----
